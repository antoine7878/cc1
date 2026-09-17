use std::fmt::{self, Display, Formatter};

use crate::ast::{ConstValue, StringConstId, StringId};
use crate::semantic::{Diagnosis, Linkage, SymbolId};
#[derive(Debug, Clone, Copy)]
pub enum LlvmName {
    SSA(usize),
    Constant(ConstValue),
    StringLiteral(StringConstId),
    Bool(bool),
    Label(usize),
    NamedLabel(StringId),
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
    pub fn label(i: usize) -> LlvmName {
        Self::Label(i)
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
            LlvmName::Constant(ConstValue::LongDouble(v)) => {
                let (head, mantissa) = v.to_bits();
                write!(f, "0xK{head:04X}{mantissa:016X}")
            }
            LlvmName::Constant(value) => write!(f, "{value}"),
            LlvmName::Bool(b) => write!(f, "{b}"),
            LlvmName::StringLiteral(i) => write!(f, "@.str.{}", usize::from(*i)),
            LlvmName::Label(i) => write!(f, "%.l{i}"),
            LlvmName::NamedLabel(i) => write!(f, "%.ln.{}", i.resolve()),
            LlvmName::Global(s) => {
                let sym = s.resolve();
                match sym.linkage {
                    Linkage::None => write!(f, "@{}.{}", sym.name.id.resolve(), usize::from(*s)),
                    _ => write!(f, "@{}", sym.name.id.resolve()),
                }
            }
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
