use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Block {
    pub hash: String,
    pub parent_hash: String,
    pub weight: u64, // not needed
}

#[derive(Clone, Debug)]
pub struct Blockchain {
    blocks: HashMap<String, Block>,
    children: HashMap<String, Vec<String>>,
}

impl Blockchain {
    pub fn new(genesis: Block) -> Self {
        let mut bc = Self { blocks: HashMap::new(), children: HashMap::new() };
        bc.children.insert(genesis.hash.clone(), Vec::new());
        bc.blocks.insert(genesis.hash.clone(), genesis);
        bc
    }

    pub fn insert(&mut self, b: Block) {
        self.children.entry(b.hash.clone()).or_default();
        self.children.entry(b.parent_hash.clone()).or_default().push(b.hash.clone());
        self.blocks.insert(b.hash.clone(), b);
    }

    pub fn get_block(&self, h: &str) -> Option<&Block> {
        self.blocks.get(h)
    }

    pub fn get_children(&self, h: &str) -> Vec<String> {
        self.children.get(h).cloned().unwrap_or_default()
    }

    fn child_for_vote(&self, root: &str, vote: &str) -> Option<String> {
        let mut cur = vote;
        while let Some(b) = self.get_block(cur) {
            if b.parent_hash == root {
                return Some(cur.to_string());
            }
            if b.parent_hash.is_empty() {
                break;
            }
            cur = &b.parent_hash;
        }
        None
    }
}

/// LMD-GHOST based ForkChoice algo
pub struct ForkChoice<'a> {
    bc: &'a Blockchain,             // References blockchain
    votes: HashMap<u64, String>,   // sumappina latest validatoriu su block kuriam dave vote
    weights: HashMap<u64, u64>,    // validator -> weight (stake)
    justified: String,             // starting point (justified checkpoint)
}

impl<'a> ForkChoice<'a> {
    pub fn new(bc: &'a Blockchain, justified: String) -> Self {
        Self { bc, votes: HashMap::new(), weights: HashMap::new(), justified }
    }       // Sukuria nauja ForkChoice instance su tusciais HashMapais ir duotu justified starting point

    pub fn cast_vote(&mut self, vid: u64, block: String) {
        self.votes.insert(vid, block);                  // Issaugo naujausia validator vote
    }

    pub fn set_weight(&mut self, vid: u64, w: u64) {
        self.weights.insert(vid, w);                    // Paskyriam stake svori validatoriui
    }

    /// ForkChoice eiga
    pub fn head(&self) -> String {
        let mut current = self.justified.clone();           // Pradedam nuo justified starting point

        loop {                                                      // Pereinam per visus justified vaikinius blokus
            let children = self.bc.get_children(&current);
            if children.is_empty() {
                break;
            }

            // Sumappinam vaikiniu bloku votes              
            let mut counts: HashMap<String, u128> =
                children.iter().map(|c| (c.clone(), 0)).collect();

            // Susumuojam svorius
            for (vid, voted) in &self.votes {
                if let Some(child) = self.bc.child_for_vote(&current, voted) {
                    let w = *self.weights.get(vid).unwrap_or(&1) as u128;
                    if let Some(cnt) = counts.get_mut(&child) {
                        *cnt += w;
                    }
                }
            }

            // Isrenkam "Laimetoja" vaikini bloka ir viska loopinam is naujo, nustate ta nugaletoja kaip current bloka
            let mut best = None;
            let mut best_v = 0;
            for (child, v) in counts {
                if best.is_none() || v > best_v || (v == best_v && child < *best.as_ref().unwrap()) {
                    best = Some(child);
                    best_v = v;
                }
            }

            if let Some(ch) = best {
                if best_v == 0 {
                    break; 
                }
                current = ch;
            } else {
                break;
            }
        }

        current
    }
}

// demo
fn main() {
    let mut bc = Blockchain::new(Block { hash: "GENESIS".into(), parent_hash: "".into(), weight: 0 });
    bc.insert(Block { hash: "A".into(), parent_hash: "GENESIS".into(), weight: 0 });
    bc.insert(Block { hash: "A1".into(), parent_hash: "A".into(), weight: 0 });
    bc.insert(Block { hash: "B".into(), parent_hash: "GENESIS".into(), weight: 0 });
    bc.insert(Block { hash: "B1".into(), parent_hash: "B".into(), weight: 0 });

    let mut fc = ForkChoice::new(&bc, "GENESIS".into());
    fc.cast_vote(1, "A1".into());
    fc.cast_vote(2, "A1".into());
    fc.cast_vote(3, "B1".into());
    fc.set_weight(1, 1);
    fc.set_weight(2, 1);
    fc.set_weight(3, 1);

    println!("LMD-GHOST head: {}", fc.head());
}
