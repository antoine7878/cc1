mod arena;
mod id;
mod interner;

pub use arena::Arena;
pub use id::{ArenaId, ArenaKey};
pub use interner::Interner;

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
