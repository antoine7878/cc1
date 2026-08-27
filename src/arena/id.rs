use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

pub trait ArenaKey: From<usize> + Into<usize> + Copy + Debug + PartialEq + Eq {}

impl<T> ArenaKey for T where T: From<usize> + Into<usize> + Copy + Debug + PartialEq + Eq {}

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
