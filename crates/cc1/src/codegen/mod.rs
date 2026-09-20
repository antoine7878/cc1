pub mod abi;
pub mod bitfield;
pub mod expression;
pub mod generator;
pub mod global;
pub mod llvm;
pub mod local;
pub mod statement;

pub use abi::{ParamAttr, ReturnAttr, classify_param};
pub use bitfield::BitField;
pub use generator::{Generator, generate, generate_to};
pub use global::Globals;
pub use llvm::{
    Builder, LlvmElement, LlvmInit, LlvmName, LlvmOperator, LlvmParam, LlvmSymbol, LlvmType, struct_elements,
};
pub use local::Locals;

use crate::arena::{ArenaId, Has};
use crate::semantic::{Diagnostic, Sema, sema};

pub trait Frozen<T> {
    fn resolve(self) -> &'static T;
}

impl<T: 'static> Frozen<T> for ArenaId<T>
where
    Sema: Has<T>,
{
    fn resolve(self) -> &'static T {
        sema().get(self)
    }
}

pub trait Invariant<T> {
    fn invariant(self, what: &'static str) -> Result<T, Diagnostic>;
}

impl<T> Invariant<T> for Option<T> {
    fn invariant(self, what: &'static str) -> Result<T, Diagnostic> {
        self.ok_or(Diagnostic::Invariant(what))
    }
}
