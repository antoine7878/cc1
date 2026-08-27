use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

#[macro_export]
macro_rules! define_arena {
    ($ty:ident, $arena:ident, $id:ident, $($ar:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Arena<$id, $ty>;

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::parser::Context) -> &'a $ty {
                ctx.$($ar).+.get(self.clone())
            }
        }
    };
}

#[macro_export]
macro_rules! define_interner {
    ($ty:ident, $arena:ident, $id:ident, $($ar:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Interner<$id, $ty>;

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::parser::Context) -> &'a $ty {
                ctx.$($ar).+.get(self.clone())
            }
        }
    };
}

#[repr(transparent)]
pub struct ArenaId<T>(u32, PhantomData<fn() -> T>);

impl<T> PartialEq for ArenaId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ArenaId<T> {}

impl<T> Hash for ArenaId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> From<ArenaId<T>> for usize {
    fn from(value: ArenaId<T>) -> usize {
        value.0 as usize
    }
}

impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ArenaId<T> {}

impl<T> From<usize> for ArenaId<T> {
    fn from(value: usize) -> ArenaId<T> {
        Self(value as u32, PhantomData)
    }
}

impl<T> Display for ArenaId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}Id({})",
            std::any::type_name::<T>().rsplit("::").next().unwrap_or("?"),
            self.0
        )
    }
}

impl<T> Debug for ArenaId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}Id({})",
            std::any::type_name::<T>().rsplit("::").next().unwrap_or("?"),
            self.0
        )
    }
}

#[derive(Debug)]
pub struct Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
{
    pub data: Vec<Val>,
    marker: PhantomData<fn() -> Id>,
}

impl<Id, Val> Default for Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
{
    fn default() -> Self {
        Self {
            data: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Id, Val> Arena<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
{
    pub fn alloc(&mut self, value: Val) -> Id {
        let id = Id::from(self.data.len());
        self.data.push(value);
        id
    }

    pub fn get(&self, id: Id) -> &Val {
        &self.data[id.into()]
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
}

#[derive(Debug)]
pub struct Interner<Id, Val>
where
    Id: From<usize> + Into<usize> + Clone + Copy + Debug + PartialEq + Eq,
    Val: Hash + Eq + PartialEq,
{
    pub data: Vec<Val>,
    canonical: HashMap<Val, Id>,
}

impl<Id, Val> Default for Interner<Id, Val>
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

impl<Id, Val> Interner<Id, Val>
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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
