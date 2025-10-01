use std::collections::HashMap;

use crate::types::*;


// Store is a struct, used to track relevant information which is needed for fork-choice algorithm to make desicions.
#[derive(Debug, Default)]
pub struct Store {
    pub time: Interval,             // Current time in intervals
    pub config: Config,             // Network configuration
    pub head: Root,                 // Current cannonical chain latest block (has most the most support from validators)
    pub safe_target: Root,          // Safe target is a potential candidate which has minimium requirement of >2/3 validators support( need to read on this more ???)
    pub latest_justified: Checkpoint,   // Changes slowly (on epoch boundaries), it is a block that 2/3 validators have voted for, but not yet finalzied. 
    pub latest_finalized: Checkpoint,   // Even slower change proccess, it is a block that has been already finalized and is mathematically impossible to revert.
    pub blocks: HashMap<Root, Block>,   // Stores all blocks???
    pub states: HashMap<Root, State>,   // Stores blockchain state after executing each block
    pub latest_known_votes: HashMap<ValidatorIndex, Checkpoint>, // Keeps track of votes that have been already included in blockchain
    pub latest_new_votes: HashMap<ValidatorIndex, Checkpoint>, // Keeps track of votes that have been newly added, but not yet confirmed
}

// Constructor for field initialization
impl Store {
    pub fn new(anchor_state: State, anchor_block: Block, anchor_root: Root, anchor_slot: Slot, intervals_per_slot: u64) -> Self {
        Store {
            time: anchor_slot * intervals_per_slot,
            config: anchor_state.config.clone(),
            head: anchor_root.clone(),
            safe_target: anchor_root.clone(),
            latest_justified: anchor_state.latest_justified.clone(),
            latest_finalized: anchor_state.latest_finalized.clone(),
            blocks: HashMap::from([(anchor_root.clone(), anchor_block)]),
            states: HashMap::from([(anchor_root, anchor_state)]),
            latest_known_votes: HashMap::new(),
            latest_new_votes: HashMap::new(),
        }
    }
}
