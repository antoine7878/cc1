use std::any::type_name;
use std::fmt::Debug;
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
        Self { data: Vec::new(), marker: PhantomData }
    }
}

impl<Id: ArenaKey, Val> Arena<Id, Val> {
    pub fn alloc(&mut self, value: Val) -> Id {
        let id = Id::from(self.data.len());
        self.data.push(value);
        id
    }

    pub fn get(&self, id: Id) -> &Val {
        let len = self.data.len();
        match self.data.get(id.into()) {
            Some(value) => value,
            None => out_of_bounds::<Id, Val>("arena", id, len),
        }
    }

    pub fn try_get(&self, id: Id) -> Option<&Val> {
        self.data.get(id.into())
    }

    pub fn get_mut(&mut self, id: Id) -> &mut Val {
        let len = self.data.len();
        match self.data.get_mut(id.into()) {
            Some(value) => value,
            None => out_of_bounds::<Id, Val>("arena", id, len),
        }
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

#[cold]
#[inline(never)]
pub fn out_of_bounds<Id: Debug, Val>(kind: &str, id: Id, len: usize) -> ! {
    let ty = type_name::<Val>().rsplit("::").next().unwrap_or("?");
    panic!("{kind} {ty}: {id:?} out of bounds (len {len})")
}

#[cold]
#[inline(never)]
pub fn not_known<Id: Debug, Val>(kind: &str, id: Id) -> ! {
    let ty = type_name::<Val>().rsplit("::").next().unwrap_or("?");
    panic!("{kind} {ty}: {id:?} not known")
}
