mod arena;
mod id;
mod interner;

pub use arena::Arena;
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

pub trait Provide<H> {
    fn provide(&self) -> &H;
}

pub trait ProvideMut<H> {
    fn provide_mut(&mut self) -> &mut H;
}

#[macro_export]
macro_rules! define_arena {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $($field:ident).+) => {
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

        impl $crate::arena::ResolveWith<$crate::parser::Context> for $id {
            type Output = $ty;
            fn resolve(self, ctx: &$crate::parser::Context) -> &$ty {
                $crate::arena::ResolveWith::<$holder>::resolve(
                    self,
                    $crate::arena::Provide::<$holder>::provide(ctx),
                )
            }
        }

        impl $crate::arena::ResolveMutWith<$crate::parser::Context> for $id {
            type Output = $ty;
            fn resolve_mut(self, ctx: &mut $crate::parser::Context) -> &mut $ty {
                $crate::arena::ResolveMutWith::<$holder>::resolve_mut(
                    self,
                    $crate::arena::ProvideMut::<$holder>::provide_mut(ctx),
                )
            }
        }

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::parser::Context) -> &'a $ty {
                $crate::arena::ResolveWith::<$crate::parser::Context>::resolve(*self, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! define_interner {
    ($ty:ident, $arena:ident, $id:ident, $holder:ty, $($field:ident).+) => {
        pub type $id = $crate::arena::ArenaId<$ty>;
        pub type $arena = $crate::arena::Interner<$id, $ty>;

        impl $crate::arena::ResolveWith<$holder> for $id {
            type Output = $ty;
            fn resolve(self, holder: &$holder) -> &$ty {
                holder.$($field).+.get(self)
            }
        }

        impl $crate::arena::ResolveWith<$crate::parser::Context> for $id {
            type Output = $ty;
            fn resolve(self, ctx: &$crate::parser::Context) -> &$ty {
                $crate::arena::ResolveWith::<$holder>::resolve(
                    self,
                    $crate::arena::Provide::<$holder>::provide(ctx),
                )
            }
        }

        impl $id {
            pub fn resolve<'a>(&self, ctx: &'a $crate::parser::Context) -> &'a $ty {
                $crate::arena::ResolveWith::<$crate::parser::Context>::resolve(*self, ctx)
            }
        }
    };
}
