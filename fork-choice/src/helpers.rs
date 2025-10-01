use crate::types::*;
use std::collections::HashMap;

pub fn get_latest_justified(states: &HashMap<Root, State>) -> Option<Checkpoint> {
    states
        .values()
        .max_by_key(|state| state.latest_justified.slot)
        .map(|s| s.latest_justified.clone())
}

pub fn get_fork_choice_head(
    blocks: &HashMap<Root, Block>,
    mut root: Root,
    latest_votes: &HashMap<ValidatorIndex, Checkpoint>,
    min_score: usize,
) -> Root {
    if root == ZERO_HASH {
        root = blocks
            .iter()
            .min_by_key(|(_, block)| block.slot)
            .map(|(r, _)| *r)
            .expect("blocks must not be empty");
    }

    let mut vote_weights: HashMap<Root, usize> = HashMap::new();

    for vote in latest_votes.values() {
        if !blocks.contains_key(&vote.root) {
            continue;
        }

        let mut current = vote.root;
        while blocks[&current].slot > blocks[&root].slot {
            *vote_weights.entry(current).or_insert(0) += 1;
            current = blocks[&current].parent_root;
        }
    }

    let mut children_map: HashMap<Root, Vec<Root>> = HashMap::new();
    for (block_hash, block) in blocks {
        let weight = vote_weights.get(block_hash).copied().unwrap_or(0);
        if weight >= min_score && block.parent_root != ZERO_HASH {
            children_map.entry(block.parent_root).or_default().push(*block_hash);
        }
    }

    let mut current = root;
    loop {
        let children = match children_map.get(&current) {
            Some(list) if !list.is_empty() => list,
            _ => return current,
        };

        current = *children
            .iter()
            .max_by(|a, b| {
                let wa = vote_weights.get(*a).copied().unwrap_or(0);
                let wb = vote_weights.get(*b).copied().unwrap_or(0);
                wa.cmp(&wb)
                    .then_with(|| blocks[*a].slot.cmp(&blocks[*b].slot))
                    .then_with(|| (*a).cmp(*b))
            })
            .unwrap();
    }
}