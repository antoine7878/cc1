use std::iter::zip;

use crate::models::{Configuration, ProductionId, TokenSet};

#[derive(Debug, Clone)]
pub struct State {
    pub sets: Vec<Configuration>,
}

impl State {
    pub fn new(sets: Vec<Configuration>) -> Self {
        Self { sets }
    }

    pub fn insert_or_merge(sets: &mut Vec<Configuration>, core: (ProductionId, usize), lookaheads: &TokenSet) -> bool {
        let pos = sets.binary_search_by(|p| p.production_id.cmp(&core.0).then(p.dot_position.cmp(&core.1)));
        match pos {
            Ok(i) => sets[i].lookaheads.union_changed(lookaheads),
            Err(i) => {
                sets.insert(i, Configuration::new(core.0, core.1, lookaheads.clone()));
                true
            }
        }
    }

    pub fn merge(s1: &[Configuration], s2: &[Configuration]) -> Vec<Configuration> {
        zip(s1, s2).map(|(c1, c2)| Configuration::merge(c1, c2)).collect()
    }
}
