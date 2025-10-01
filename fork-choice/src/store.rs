use crate::types::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Store {
    pub time: Interval,
    pub head: Root,
    pub safe_target: Root,
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
    pub blocks: HashMap<Root, Block>,
    pub states: HashMap<Root, State>,
    pub latest_known_votes: HashMap<ValidatorIndex, Checkpoint>,
    pub latest_new_votes: HashMap<ValidatorIndex, Checkpoint>,
}

pub fn get_forkchoice_store(anchor_state: State, anchor_block: Block) -> Store {
    let block_root = anchor_block.state_root; // PLACEHOLDER! REPLACE!
    let time = anchor_block.slot * 8;

    Store {
        time,
        head: block_root,
        safe_target: block_root,
        latest_justified: anchor_state.latest_justified.clone(),
        latest_finalized: anchor_state.latest_finalized.clone(),
        blocks: [(block_root, anchor_block)].into(),
        states: [(block_root, anchor_state)].into(),
        latest_known_votes: HashMap::new(),
        latest_new_votes: HashMap::new(),
    }
}