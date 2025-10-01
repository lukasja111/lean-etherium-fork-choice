pub type Root = [u8; 32];
pub type Slot = u64;
pub type ValidatorIndex = u64;
pub type Interval = u64;

pub const ZERO_HASH: Root = [0u8; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub slot: Slot,
    pub root: Root,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub slot: Slot,
    pub parent_root: Root,
    pub state_root: Root,
}

#[derive(Debug, Clone)]
pub struct State {
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
}

#[derive(Debug, Clone)]
pub struct SignedVote {
    pub validator_id: ValidatorIndex,
    pub message: Checkpoint,
}