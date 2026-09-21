use std::fmt::{self, Display, Formatter};

use crate::ast::{ConstValue, F80};
use crate::codegen::{LlvmName, LlvmType};
use crate::semantic::{QualifiedType, sema};

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
        Self { name: ConstValue::UnsignedLong(idx).llvm(), ty: LlvmType::int() }
    }

    pub fn ptr(name: LlvmName) -> Self {
        Self { name, ty: LlvmType::ptr() }
    }

    pub fn cst(ty: LlvmType, value: ConstValue) -> Self {
        Self { ty, name: LlvmName::Constant(value) }
    }
    pub fn void() -> Self {
        Self { ty: LlvmType::void(), name: LlvmName::None }
    }

    pub fn null() -> Self {
        Self::ptr(LlvmName::Null)
    }

    pub fn zero(qty: QualifiedType) -> Self {
        let sema = sema();
        if qty.is_pointer(sema) {
            Self::null()
        } else if qty.is_floating(sema) {
            Self::floating(qty.llvm(), 0.0)
        } else {
            Self::cst(qty.llvm(), ConstValue::Int(0))
        }
    }

    pub fn one(qty: QualifiedType) -> Self {
        if qty.is_floating(sema()) {
            Self::floating(qty.llvm(), 1.0)
        } else {
            Self::cst(LlvmType::int(), ConstValue::Int(1))
        }
    }

    fn floating(ty: LlvmType, value: f64) -> Self {
        let value = match ty {
            LlvmType::F32 => ConstValue::Float(value as f32),
            LlvmType::F80 => ConstValue::LongDouble(F80::from(value)),
            _ => ConstValue::Double(value),
        };
        Self::cst(ty, value)
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
        write!(f, "{} {}", self.ty, self.name)
    }
}
