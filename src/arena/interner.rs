use std::collections::HashMap;
use std::hash::Hash;

use crate::arena::{Arena, ArenaKey};

#[derive(Debug)]
pub struct Interner<Id: ArenaKey, Val> {
    arena: Arena<Id, Val>,
    canonical: HashMap<Val, Id>,
}

impl<Id: ArenaKey, Val> Default for Interner<Id, Val> {
    fn default() -> Self {
        Self {
            arena: Arena::default(),
            canonical: HashMap::new(),
        }
    }
}

impl<Id: ArenaKey, Val: Clone + Hash + Eq> Interner<Id, Val> {
    pub fn alloc(&mut self, value: Val) -> Id {
        if let Some(&id) = self.canonical.get(&value) {
            return id;
        }

        let id = self.arena.alloc(value.clone());
        self.canonical.insert(value, id);
        id
    }

    pub fn get(&self, id: Id) -> &Val {
        self.arena.get(id)
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }
}
