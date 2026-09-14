pub mod expression;
pub mod generator;
pub mod global;
pub mod llvm;
pub mod local;

pub use generator::{Generator, generate, generate_to};
pub use global::Globals;
pub use llvm::{Builder, LlvmInit, LlvmName, LlvmOperator, LlvmSymbol, LlvmType};
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
