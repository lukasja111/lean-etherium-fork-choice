use std::collections::HashMap;

// This is dummy structure of types used in fork-choice for testing of fork-choice (Real types we will get from teams implementing other client parts)
pub type Interval = u64;
pub type Root = String; // No hashing for testing purposes (Might need to add later)
pub type Slot = u64;
pub type ValidatorIndex = u64;

#[derive(Clone, Debug)]
pub struct Config {
    pub num_validators: usize,
    pub genesis_time: u64,
}

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub root: Root,
    pub slot: Slot,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub slot: Slot,
    pub parent_root: Root,
    pub state_root: Root,
    pub body: BlockBody,
}

#[derive(Clone, Debug)]
pub struct BlockBody {
    pub attestations: Vec<SignedVote>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
}

#[derive(Clone, Debug)]
pub struct SignedVote {
    pub validator_id: ValidatorIndex,
    pub message: Checkpoint,
}
