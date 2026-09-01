mod id;
mod interner;
mod store;

pub use store::Arena;
pub use id::{ArenaId, ArenaKey};
pub use interner::Interner;

pub trait ResolveWith<H> {
    type Output;
    fn resolve(self, holder: &H) -> &Self::Output;
}

pub trait ResolveMutWith<H> {
    type Output;
    fn resolve_mut(self, holder: &mut H) -> &mut Self::Output;
}

#[macro_export]
macro_rules! define_arena {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $ctx_field:ident, $($field:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Arena<$id, $ty>;

        impl $crate::arena::ResolveWith<$holder> for $id {
            type Output = $ty;
            fn resolve(self, holder: &$holder) -> &$ty {
                holder.$($field).+.get(self)
            }
        }

        impl $crate::arena::ResolveMutWith<$holder> for $id {
            type Output = $ty;
            fn resolve_mut(self, holder: &mut $holder) -> &mut $ty {
                holder.$($field).+.get_mut(self)
            }
        }

        impl $crate::arena::ResolveWith<$crate::context::Context> for $id {
            type Output = $ty;
            fn resolve(self, ctx: &$crate::context::Context) -> &$ty {
                ctx.$ctx_field.$($field).+.get(self)
            }
        }

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::context::Context) -> &'a $ty {
                $crate::arena::ResolveWith::<$crate::context::Context>::resolve(*self, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! define_interner {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $ctx_field:ident, $($field:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Interner<$id, $ty>;

        impl $crate::arena::ResolveWith<$holder> for $id {
            type Output = $ty;
            fn resolve(self, holder: &$holder) -> &$ty {
                holder.$($field).+.get(self)
            }
        }

        impl $crate::arena::ResolveWith<$crate::context::Context> for $id {
            type Output = $ty;
            fn resolve(self, ctx: &$crate::context::Context) -> &$ty {
                ctx.$ctx_field.$($field).+.get(self)
            }
        }

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::context::Context) -> &'a $ty {
                $crate::arena::ResolveWith::<$crate::context::Context>::resolve(*self, ctx)
            }
        }
    };
}
