use std::collections::HashMap;
use std::hash::Hash;
use std::slice;

use crate::arena::store::out_of_bounds;
use crate::arena::{Arena, ArenaKey};

#[derive(Debug)]
pub struct Interner<Id: ArenaKey, Val> {
    arena: Arena<Id, Val>,
    canonical: HashMap<Val, Id>,
}

impl<Id: ArenaKey, Val> Default for Interner<Id, Val> {
    fn default() -> Self {
        Self { arena: Arena::default(), canonical: HashMap::new() }
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
        match self.arena.try_get(id) {
            Some(value) => value,
            None => out_of_bounds::<Id, Val>("interner", id, self.arena.len()),
        }
    }

    pub fn try_get(&self, id: Id) -> Option<&Val> {
        self.arena.try_get(id)
    }

    pub fn lookup(&self, value: &Val) -> Option<Id> {
        self.canonical.get(value).copied()
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    pub fn iter(&self) -> slice::Iter<'_, Val> {
        self.arena.iter()
    }
}
