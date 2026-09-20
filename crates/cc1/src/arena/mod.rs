mod global;
mod id;
mod interner;
mod side_table;
mod store;

pub use global::Global;
pub use id::{ArenaId, ArenaKey};
pub use interner::Interner;
pub use side_table::{HasTable, Loan, OptionPoisoned, SideTable, Slot};
pub use store::Arena;

pub trait Installed {
    fn installed() -> &'static Self;
}

pub trait Has<T> {
    fn get(&self, id: ArenaId<T>) -> &T;
}

pub trait HasMut<T>: Has<T> {
    fn get_mut(&mut self, id: ArenaId<T>) -> &mut T;
}

pub trait Owned: Sized {
    type Holder: Has<Self> + Installed;
}

#[macro_export]
macro_rules! define_arena {
    ($ty:ident, $arena:ident, $id:ident) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Arena<$id, $ty>;
    };
}

#[macro_export]
macro_rules! define_interner {
    ($ty:ident, $arena:ident, $id:ident) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Interner<$id, $ty>;
    };
}
