mod id;
mod interner;
mod side_table;
mod store;

pub use id::{ArenaId, ArenaKey};
pub use interner::Interner;
pub use side_table::{HasTable, Loan, OptionPoisoned, SideTable, Slot};
pub use store::Arena;

pub trait ResolveWith<H> {
    type Output;
    fn resolve_in(self, holder: &H) -> &Self::Output;
}

pub trait ResolveMutWith<H> {
    type Output;
    fn resolve_mut(self, holder: &mut H) -> &mut Self::Output;
}

#[macro_export]
macro_rules! define_arena {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $global:expr, $($field:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Arena<$id, $ty>;

        impl $crate::arena::ResolveWith<$holder> for $id {
            type Output = $ty;
            fn resolve_in(self, holder: &$holder) -> &$ty {
                holder.$($field).+.get(self)
            }
        }

        impl $crate::arena::ResolveMutWith<$holder> for $id {
            type Output = $ty;
            fn resolve_mut(self, holder: &mut $holder) -> &mut $ty {
                holder.$($field).+.get_mut(self)
            }
        }

        impl $id {
            pub fn resolve(self) -> &'static $ty {
                $global.$($field).+.get(self)
            }
        }
    };
}

#[macro_export]
macro_rules! define_interner {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $global:expr, $($field:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Interner<$id, $ty>;

        impl $crate::arena::ResolveWith<$holder> for $id {
            type Output = $ty;
            fn resolve_in(self, holder: &$holder) -> &$ty {
                holder.$($field).+.get(self)
            }
        }

        impl $id {
            pub fn resolve(self) -> &'static $ty {
                $global.$($field).+.get(self)
            }
        }
    };
}
