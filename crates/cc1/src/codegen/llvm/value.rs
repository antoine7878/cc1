use std::fmt::{self, Display, Formatter};

use crate::ast::ConstValue;
use crate::semantic::SymbolId;

#[derive(Debug, Clone, Copy)]
pub enum LlvmName {
    SSA(usize),
    Constant(ConstValue),
    StringLiteral(usize),
    Bool(bool),
    Label(usize, usize),
    Global(SymbolId),
}

impl ConstValue {
    pub fn llvm(&self) -> LlvmName {
        LlvmName::Constant(*self)
    }
}

impl LlvmName {
    pub fn label(i: usize, j: usize) -> LlvmName {
        Self::Label(i, j)
    }
}

impl Display for LlvmName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmName::SSA(id) => write!(f, "%{id}"),
            LlvmName::Constant(value) => write!(f, "{value}"),
            LlvmName::Bool(b) => write!(f, "{b}"),
            LlvmName::StringLiteral(i) => write!(f, "@.str.{i}"),
            LlvmName::Label(i, j) => write!(f, "%l.{i}.{j}"),
            LlvmName::Global(s) => write!(f, "@{}", s.resolve().name.id.resolve()),
        }
    }
}

impl From<bool> for LlvmName {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
