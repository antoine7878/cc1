pub mod expression;
pub mod generator;
pub mod global;
pub mod llvm;
pub mod local;
pub mod statement;

pub use generator::{Generator, generate, generate_to};
pub use global::Globals;
pub use llvm::{Builder, LlvmElement, LlvmInit, LlvmName, LlvmOperator, LlvmSymbol, LlvmType, struct_elements};
pub use local::Locals;

use crate::semantic::Diagnosis;

pub trait Invariant<T> {
    fn invariant(self, what: &'static str) -> Result<T, Diagnosis>;
}

impl<T> Invariant<T> for Option<T> {
    fn invariant(self, what: &'static str) -> Result<T, Diagnosis> {
        self.ok_or(Diagnosis::Invariant(what))
    }
}
