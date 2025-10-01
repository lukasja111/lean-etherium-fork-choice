use fork_choice::*;

#[test]
fn test_genesis_store() {
    let genesis_state = State {
        latest_justified: Checkpoint {
            slot: 0,
            root: ZERO_HASH,
        },
        latest_finalized: Checkpoint {
            slot: 0,
            root: ZERO_HASH,
        },
    };

    let genesis_block = Block {
        slot: 0,
        parent_root: ZERO_HASH,
        state_root: [1u8; 32],
    };

    let store = get_forkchoice_store(genesis_state, genesis_block);
    assert_eq!(store.head, [1u8; 32]);
    assert_eq!(store.safe_target, [1u8; 32]);
}

#[test]
fn test_add_block_and_update_head() {
    let genesis_state = State {
        latest_justified: Checkpoint {
            slot: 0,
            root: ZERO_HASH,
        },
        latest_finalized: Checkpoint {
            slot: 0,
            root: ZERO_HASH,
        },
    };

    let genesis_block = Block {
        slot: 0,
        parent_root: ZERO_HASH,
        state_root: [1u8; 32],
    };

    let mut store = get_forkchoice_store(genesis_state, genesis_block);

    //add block at slot 1
    let block1 = Block {
        slot: 1,
        parent_root: [1u8; 32],
        state_root: [2u8; 32],
    };

    on_block(&mut store, [2u8; 32], block1);

    //head should now be block1
    assert_eq!(store.head, [2u8; 32]);
}