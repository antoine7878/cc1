use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::models::{ProductionId, TokenSet};
use crate::utils::BitSet;

#[derive(Debug, Clone, Eq)]
pub struct Configuration {
    pub production_id: ProductionId,
    pub dot_position: usize,
    pub lookaheads: TokenSet, /* TokenId */
}

impl Configuration {
    pub fn new(production_id: ProductionId, dot_position: usize, lookaheads: BitSet) -> Self {
        Self {
            production_id,
            dot_position,
            lookaheads,
        }
    }

    pub fn next(&self) -> Self {
        Self::new(self.production_id, self.dot_position + 1, self.lookaheads.clone())
    }

    pub fn merge(c1: &Configuration, c2: &Configuration) -> Configuration {
        let mut lookaheads = c1.lookaheads.clone();
        lookaheads.union_with(&c2.lookaheads);
        Configuration::new(c2.production_id, c2.dot_position, lookaheads)
    }

    pub fn core(&self) -> (ProductionId, usize) {
        (self.production_id, self.dot_position)
    }
}

impl PartialOrd for Configuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Configuration {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.production_id.cmp(&other.production_id);
        let b = self.dot_position.cmp(&other.dot_position);
        let c = self.lookaheads.cmp(&other.lookaheads);
        a.then(b).then(c)
    }
}

impl PartialEq for Configuration {
    fn eq(&self, other: &Self) -> bool {
        self.production_id == other.production_id
            && self.dot_position == other.dot_position
            && self.lookaheads == other.lookaheads
    }
}

impl Hash for Configuration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.production_id.hash(state);
        self.dot_position.hash(state);
        self.lookaheads.hash(state);
    }
}
