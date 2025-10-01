use crate::store::Store;
use crate::types::*;
use crate::utils::{SECONDS_PER_INTERVAL, INTERVALS_PER_SLOT};

pub fn on_tick(store: &mut Store, time: u64, has_proposal: bool) {          // Time Advancement handler (Function is called periodically by the node, before block proposal, after network sync)
    let tick_interval_time = (time - store.config.genesis_time) / SECONDS_PER_INTERVAL;

    while store.time < tick_interval_time {
        let should_signal_proposal = has_proposal && (store.time + 1) == tick_interval_time;
        crate::helpers::tick_interval(store, should_signal_proposal);
    }
}


// validate_on_attestation - need to implement this function, for now on_attestation function assumes that votes are always valid
pub fn on_attestation(store: &mut Store, signed_vote: SignedVote, is_from_block: bool) {
    let validator_id = signed_vote.validator_id;
    let vote = signed_vote.message;

    if is_from_block {

        if let Some(latest_vote) = store.latest_known_votes.get(&validator_id) {
            if latest_vote.slot < vote.slot {           // Update known votes (cannonical on-chain vote history, can only be appended not decreased)
                store.latest_known_votes.insert(validator_id, vote.clone());
            }
        } else {
            store.latest_known_votes.insert(validator_id, vote.clone());
        }

        // 
        if let Some(latest_new_vote) = store.latest_new_votes.get(&validator_id) {  // Pending and new votes, which can be either later confirmed or removed
            if latest_new_vote.slot < vote.slot {
                store.latest_new_votes.remove(&validator_id);
            }
        }
    } else {
        // For gossiped votes, ensure forkchoice correct ticking
        let time_slots = store.time; // 
        assert!(vote.slot <= time_slots, "Vote slot is in the future");         // prevent voting for future

        // Update latest new votes if this is the latest
        if let Some(latest_vote) = store.latest_new_votes.get(&validator_id) {
            if latest_vote.slot < vote.slot {
                store.latest_new_votes.insert(validator_id, vote);
            }
        } else {
            store.latest_new_votes.insert(validator_id, vote);
        }
    }
}

pub fn on_block(store: &mut Store, block: Block) {
    let block_hash = format!("block_{}", block.slot); // Simplified for testing, would need to implement
    
    // If block was already proccessed ignore it
    if store.blocks.contains_key(&block_hash) {
        return;
    }

    let parent_state = store.states.get(&block.parent_root);        // Check if the block in quesetion has a parent, this prevents orphan blocks from affecting fork-choice
    assert!(parent_state.is_some(), "Parent state not found, sync parent chain first");

    // Simplified state transition, here we will need to call state transition function, which other team will provide
    if let Some(parent_state) = parent_state {
        let mut new_state = parent_state.clone();
        
        // Update state based on the new block 
        new_state.latest_justified = Checkpoint {
            root: block_hash.clone(),
            slot: block.slot,
        };
        
        if block.slot > new_state.latest_finalized.slot + 1 {
            new_state.latest_finalized = Checkpoint {
                root: block.parent_root.clone(),
                slot: block.slot - 1,
            };
        }

        store.blocks.insert(block_hash.clone(), block.clone());
        store.states.insert(block_hash.clone(), new_state);

        // Process votes in the block
        for signed_vote in block.body.attestations {
            on_attestation(store, signed_vote, true);
        }

        crate::helpers::update_head(store);
    }
}