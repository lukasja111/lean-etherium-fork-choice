use crate::{helpers, store::Store, types::*};

const INTERVALS_PER_SLOT: Interval = 8;
const SECONDS_PER_SLOT: u64 = 12;

pub fn update_head(store: &mut Store) {
    if let Some(latest_justified) = helpers::get_latest_justified(&store.states) {
        store.latest_justified = latest_justified;
    }

    store.head = helpers::get_fork_choice_head(
        &store.blocks,
        store.latest_justified.root,
        &store.latest_known_votes,
        0,
    );

    if let Some(state) = store.states.get(&store.head) {
        store.latest_finalized = state.latest_finalized.clone();
    }
}

pub fn update_safe_target(store: &mut Store) {
    let num_validators = store.states.len().max(1);
    let min_target_score = (num_validators * 2 + 2) / 3;

    store.safe_target = helpers::get_fork_choice_head(
        &store.blocks,
        store.latest_justified.root,
        &store.latest_new_votes,
        min_target_score,
    );
}

pub fn accept_new_votes(store: &mut Store) {
    store.latest_known_votes.extend(store.latest_new_votes.drain());
    update_head(store);
}

pub fn tick_interval(store: &mut Store, has_proposal: bool) {
    store.time += 1;
    let current_interval = store.time % INTERVALS_PER_SLOT;

    match current_interval {
        0 if has_proposal => accept_new_votes(store),
        2 => update_safe_target(store),
        _ => {
            if current_interval != 1 {
                accept_new_votes(store);
            }
        }
    }
}

pub fn on_tick(store: &mut Store, time: u64, has_proposal: bool, genesis_time: u64) {
    let elapsed_intervals = time.saturating_sub(genesis_time) / (SECONDS_PER_SLOT / INTERVALS_PER_SLOT);
    while store.time < elapsed_intervals {
        let next_has_proposal = has_proposal && (store.time + 1 == elapsed_intervals);
        tick_interval(store, next_has_proposal);
    }
}

pub fn on_attestation(
    store: &mut Store,
    attestation: SignedVote,
    is_from_block: bool,
    current_time: u64,
    genesis_time: u64,
) {
    let validator_id = attestation.validator_id;
    let vote = attestation.message;

    if is_from_block {
        if store
            .latest_known_votes
            .get(&validator_id)
            .map_or(true, |v| v.slot < vote.slot)
        {
            store.latest_known_votes.insert(validator_id, vote.clone());
        }
        if let Some(existing) = store.latest_new_votes.get(&validator_id) {
            if existing.slot < vote.slot {
                store.latest_new_votes.remove(&validator_id);
            }
        }
    } else {
        let current_slot = (current_time - genesis_time) / SECONDS_PER_SLOT;
        if vote.slot > current_slot {
            return;
        }
        if store
            .latest_new_votes
            .get(&validator_id)
            .map_or(true, |v| v.slot < vote.slot)
        {
            store.latest_new_votes.insert(validator_id, vote);
        }
    }
}

pub fn on_block(store: &mut Store, block_root: Root, block: Block) {
    if store.blocks.contains_key(&block_root) {
        return;
    }

    assert!(
        store.states.contains_key(&block.parent_root),
        "Parent state missing"
    );

    let parent_state = store.states.get(&block.parent_root).unwrap().clone();
    let new_state = parent_state; 

    store.blocks.insert(block_root, block);
    store.states.insert(block_root, new_state);

    update_head(store);
}