use std::collections::HashMap;
use crate::types::*;
use crate::store::Store;

const ZERO_HASH: &str = "0";

// LMD-GHOST algorithm (Finds the chain with most validator support)
pub fn get_fork_choice_head(
    blocks: &HashMap<Root, Block>,      // All known blocks
    root: &Root,                        // Starting point
    latest_votes: &HashMap<ValidatorIndex, Checkpoint>,         // Validator votes
    min_score: Option<usize>,                                   // Minimum votes required (for safe target determination)
) -> Root {
    let min_score = min_score.unwrap_or(0);
    
    // If root is zero start at genesis, otherwise start from provided root
    let mut current_root = if root == ZERO_HASH {
        blocks.keys()
            .min_by_key(|block_hash| blocks[*block_hash].slot)
            .cloned()
            .unwrap_or_else(|| root.clone())
    } else {
        root.clone()
    };

    // Count votes for each block
    let mut vote_weights: HashMap<Root, usize> = HashMap::new();

    for vote in latest_votes.values() {
        if blocks.contains_key(&vote.root) {
            let mut block_hash = vote.root.clone();     // A -> B -> C if vote is casted for C, then vote is added on to ancestors (A and B)
            while let Some(block) = blocks.get(&block_hash) {
                if block.slot > blocks[&current_root].slot {
                    *vote_weights.entry(block_hash.clone()).or_insert(0) += 1;
                    block_hash = block.parent_root.clone();
                } else {
                    break;
                }
            }
        }
    }

    // Find children of each block
    let mut children_map: HashMap<Root, Vec<Root>> = HashMap::new();
    for (block_hash, block) in blocks {
        if block.parent_root != ZERO_HASH && vote_weights.get(block_hash).unwrap_or(&0) >= &min_score {         // We only include blocks with enough votes and non-genesis parents
            children_map
                .entry(block.parent_root.clone())
                .or_insert_with(Vec::new)
                .push(block_hash.clone());
        }
    }

    // Start at the root and in loop keep choosing child with most votes
    let mut current = current_root;
    loop {
        if let Some(children) = children_map.get(&current) {
            if children.is_empty() {        // Means that no more children, we reached the tip
                return current;
            }
            
            current = children.iter()       // Select child with most votes, if there is a tie-breaker select by higher slot, if slot is tied, select by comparing hashes (higher wins)
                .max_by_key(|child| {
                    let vote_weight = vote_weights.get(*child).unwrap_or(&0);
                    let block_slot = blocks[*child].slot;
                    (vote_weight, block_slot, *child)
                })
                .unwrap()
                .clone();
        } else {
            return current;
        }
    }
}

// Find latest justified_checkpoint (block) accross all states of blockchain
pub fn get_latest_justified(states: &HashMap<Root, State>) -> Option<Checkpoint> {
    states.values()
        .max_by_key(|state| state.latest_justified.slot)
        .map(|state| state.latest_justified.clone())
}
// Create initial fork-choice store
pub fn get_forkchoice_store(anchor_state: State, anchor_block: Block, intervals_per_slot: u64) -> Store {
    let anchor_root = "genesis".to_string(); // Simplified for testing ( need to hash this)
    
    Store::new(anchor_state, anchor_block, anchor_root.clone(), anchor_block.slot, intervals_per_slot)
}

// Calculate and update the stores latest_justified block and latest_finalized block
pub fn update_head(store: &mut Store) {
    if let Some(latest_justified) = get_latest_justified(&store.states) {
        store.latest_justified = latest_justified.clone();
    }
    
    store.head = get_fork_choice_head(
        &store.blocks,
        &store.latest_justified.root,
        &store.latest_known_votes,
        None,
    );

    if let Some(head_state) = store.states.get(&store.head) {
        store.latest_finalized = head_state.latest_finalized.clone();
    }
}

// Find safe voting target 
pub fn update_safe_target(store: &mut Store) {
    let min_target_score = (store.config.num_validators * 2 + 2) / 3; 
    
    store.safe_target = get_fork_choice_head(
        &store.blocks,
        &store.latest_justified.root,
        &store.latest_new_votes,
        Some(min_target_score),
    );
}

fn is_justifiable_slot(finalized_slot: Slot, target_slot: Slot) -> bool {
    // Need to write algorithm to identify wheter justifable block is within the justification window
    target_slot > finalized_slot
}


// Calculate and determine where validators should vote
pub fn get_vote_target(store: &Store) -> Checkpoint {
    let mut target_block_root = store.head.clone();

    // Prevents from voting too far ahead
    for _ in 0..3 {
        if let (Some(current_block), Some(safe_block)) = (
            store.blocks.get(&target_block_root),
            store.blocks.get(&store.safe_target)
        ) {
            if current_block.slot > safe_block.slot {
                target_block_root = current_block.parent_root.clone();
            }
        }
    }

    // Check if target which we are voting for is justifable
    while let Some(target_block) = store.blocks.get(&target_block_root) {
        if !is_justifiable_slot(store.latest_finalized.slot, target_block.slot) {
            target_block_root = target_block.parent_root.clone();
        } else {
            break;
        }
    }

    // Returns latest justfied block or fallback to latest justified block before the calculations if something went wrong
    if let Some(target_block) = store.blocks.get(&target_block_root) {
        Checkpoint {
            root: target_block_root,
            slot: target_block.slot,
        }
    } else {
        store.latest_justified.clone()
    }
}

// Move votes from new_votes to known_votes
pub fn accept_new_votes(store: &mut Store) {
    for (validator_id, vote) in store.latest_new_votes.drain() {
        store.latest_known_votes.insert(validator_id, vote);
    }
    update_head(store); // Recalculate head with new votes
}

// Handle time advancement and interval logic
pub fn tick_interval(store: &mut Store, has_proposal: bool) {
    store.time += 1;        // We advance by one interval
    let current_interval = store.time % crate::utils::INTERVALS_PER_SLOT;
    
    match current_interval {
        0 => {          // First interval
            if has_proposal {
                accept_new_votes(store);        // Validators will vote in this interval using safe target previously
            }
        }
        2 => {          // Third interval 
            update_safe_target(store);  // Update safe_target
        }
        _ => {
            // Don't do anything for other intervals
        }
    }
}

pub fn get_proposal_head(store: &mut Store, slot: Slot, seconds_per_slot: u64) -> Root {
    let slot_time = store.config.genesis_time + slot * seconds_per_slot;
    
    // Simulate time tick
    crate::handlers::on_tick(store, slot_time, true);
    
    accept_new_votes(store);    // Process pending votes
    store.head.clone()          // Return current head
}