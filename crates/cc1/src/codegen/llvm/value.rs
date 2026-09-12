use std::fmt::{self, Display, Formatter};

use crate::ast::ConstValue;
use crate::semantic::{Diagnosis, SymbolId};

#[derive(Debug, Clone, Copy)]
pub enum LlvmName {
    SSA(usize),
    Constant(ConstValue),
    StringLiteral(usize),
    Bool(bool),
    Label(usize, usize),
    Global(SymbolId),
    Null,
    None,
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

    pub fn ssa_value(&self) -> Result<usize, Diagnosis> {
        match self {
            LlvmName::SSA(i) => Ok(*i),
            _ => Err(Diagnosis::Invariant("not SSA name")),
        }
    }
}

impl Display for LlvmName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmName::SSA(id) => write!(f, "%{id}"),
            LlvmName::Constant(ConstValue::Float(v)) => write!(f, "0x{:016X}", f64::from(*v).to_bits()),
            LlvmName::Constant(ConstValue::Double(v)) => write!(f, "0x{:016X}", v.to_bits()),
            LlvmName::Constant(value) => write!(f, "{value}"),
            LlvmName::Bool(b) => write!(f, "{b}"),
            LlvmName::StringLiteral(i) => write!(f, "@.str.{i}"),
            LlvmName::Label(i, j) => write!(f, "%l.{i}.{j}"),
            LlvmName::Global(s) => write!(f, "@{}", s.resolve().name.id.resolve()),
            LlvmName::Null => write!(f, "null"),
            LlvmName::None => Ok(()),
        }
    }
}

impl From<bool> for LlvmName {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
