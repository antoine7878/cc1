use std::marker::PhantomData;
use std::slice;

use crate::arena::ArenaKey;

#[derive(Debug)]
pub struct Arena<Id: ArenaKey, Val> {
    data: Vec<Val>,
    marker: PhantomData<fn() -> Id>,
}

impl<Id: ArenaKey, Val> Default for Arena<Id, Val> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Id: ArenaKey, Val> Arena<Id, Val> {
    pub fn alloc(&mut self, value: Val) -> Id {
        let id = Id::from(self.data.len());
        self.data.push(value);
        id
    }

    pub fn get(&self, id: Id) -> &Val {
        &self.data[id.into()]
    }

    pub fn try_get(&self, id: Id) -> Option<&Val> {
        self.data.get(id.into())
    }

    pub fn get_mut(&mut self, id: Id) -> &mut Val {
        &mut self.data[id.into()]
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> slice::Iter<'_, Val> {
        self.data.iter()
    }
}
