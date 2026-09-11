use std::fmt::{self, Display, Formatter};

use crate::ast::ConstValue;
use crate::semantic::SymbolId;

#[derive(Debug, Clone, Copy)]
pub enum LlvmValue {
    SSA(usize),
    Constant(ConstValue),
    StringLiteral(usize),
    Bool(bool),
    Label(usize, usize),
    Global(SymbolId),
}
impl LlvmValue {
    pub fn label(i: usize, j: usize) -> LlvmValue {
        Self::Label(i, j)
    }
}

impl LlvmValue {
    pub fn zero() -> Self {
        LlvmValue::Constant(ConstValue::zero())
    }

    pub fn one() -> Self {
        LlvmValue::Constant(ConstValue::one())
    }

    pub fn minus_one() -> Self {
        LlvmValue::Constant(ConstValue::minus_one())
    }
}

impl Display for LlvmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmValue::SSA(id) => write!(f, "%{id}"),
            LlvmValue::Constant(value) => write!(f, "{value}"),
            LlvmValue::Bool(b) => write!(f, "{b}"),
            LlvmValue::Label(i, j) => write!(f, "%l.{i}.{j}"),
            LlvmValue::StringLiteral(i) => write!(f, "@.str.{i}"),
            LlvmValue::Global(s) => write!(f, "@{}", s.resolve().name.id.resolve()),
        }
    }
}

impl From<bool> for LlvmValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
