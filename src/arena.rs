use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ArenaId<T>(u32, PhantomData<T>);

impl<T> From<ArenaId<T>> for usize {
    fn from(value: ArenaId<T>) -> usize {
        value.0 as usize
    }
}

impl<T> Copy for ArenaId<T> {}

impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> From<usize> for ArenaId<T> {
    fn from(value: usize) -> ArenaId<T> {
        Self(value as u32, PhantomData)
    }
}

#[derive(Debug)]
pub struct Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
    Val: Hash + Eq + PartialEq,
{
    pub data: Vec<Val>,
    pub canonical: HashMap<Val, Id>,
}

impl<Id, Val> Default for Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
    Val: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self {
            data: Vec::new(),
            canonical: HashMap::new(),
        }
    }
}

impl<Id, Val> Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
    Val: Clone + Hash + Eq + PartialEq,
{
    pub fn alloc(&mut self, value: Val) -> Id {
        if let Some(&id) = self.canonical.get(&value) {
            return id;
        }

        let id = Id::from(self.data.len());
        self.data.push(value.clone());
        self.canonical.insert(value, id);
        id
    }

    pub fn get(&self, id: Id) -> &Val {
        &self.data[id.into()]
    }
}
