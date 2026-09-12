use std::fmt::{self, Display, Formatter};

use crate::{
    ast::ConstValue,
    codegen::{LlvmName, LlvmType, llvm::types::LlvmFirstType},
};

#[derive(Debug, Clone, Copy)]
pub struct LlvmSymbol {
    pub name: LlvmName,
    pub ty: LlvmType,
}

impl LlvmSymbol {
    pub fn new(ty: LlvmType, name: LlvmName) -> Self {
        Self { name, ty }
    }

    pub fn idx(idx: u64) -> Self {
        Self { name: ConstValue::UnsignedLong(idx).llvm(), ty: LlvmType::ptr_size() }
    }

    pub fn ptr(name: LlvmName) -> Self {
        Self { name, ty: LlvmType::ptr() }
    }

    pub fn cst(ty: LlvmType, value: ConstValue) -> Self {
        Self { ty, name: LlvmName::Constant(value) }
    }
}

impl From<ConstValue> for LlvmSymbol {
    fn from(value: ConstValue) -> Self {
        Self::cst(LlvmType::from(value), value)
    }
}

impl From<&ConstValue> for LlvmSymbol {
    fn from(value: &ConstValue) -> Self {
        LlvmSymbol::from(*value)
    }
}

impl From<bool> for LlvmSymbol {
    fn from(value: bool) -> Self {
        Self { ty: LlvmType::bool(), name: LlvmName::Bool(value) }
    }
}

impl From<i32> for LlvmSymbol {
    fn from(value: i32) -> Self {
        Self::cst(LlvmType::int(), ConstValue::Int(value))
    }
}

impl Display for LlvmSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.ty {
            LlvmType::First(LlvmFirstType::Void) => write!(f, "void"),
            _ => write!(f, "{} {}", self.ty, self.name),
        }
    }
}
