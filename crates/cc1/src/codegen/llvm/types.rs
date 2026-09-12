use std::fmt;

use crate::context::ctx;
use crate::semantic::{QualifiedType, ResolvedType};

impl ResolvedType {
    pub fn llvm(&'static self) -> LlvmType {
        LlvmType { ty: self }
    }
}

impl QualifiedType {
    pub fn llvm(&self) -> LlvmType {
        self.id.resolve().llvm()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LlvmType {
    ty: &'static ResolvedType,
}

impl LlvmType {
    pub fn bool() -> Self {
        Self { ty: &ResolvedType::Bool }
    }
}

impl fmt::Display for LlvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = &ctx().target;
        match self.ty {
            ResolvedType::Bool
            | ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::UnsignedChar
            | ResolvedType::Short
            | ResolvedType::UnsignedShort
            | ResolvedType::Long
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong
            | ResolvedType::Int => write!(f, "i{}", target.layout(self.ty).unwrap().size * 8),
            ResolvedType::Void => write!(f, "void"),
            ResolvedType::Float => write!(f, "{}", target.float.llvm()),
            ResolvedType::Double => write!(f, "{}", target.double.llvm()),
            ResolvedType::LongDouble => write!(f, "{}", target.long_double.llvm()),
            ResolvedType::Array { elem, len: Some(len) } => write!(f, "[{} x {}]", len, elem.llvm()),
            ResolvedType::Pointer(_) => write!(f, "ptr"),
            ResolvedType::Function { .. } => todo!(),
            ResolvedType::Tag { .. } => todo!(),
            ResolvedType::Array { len: None, .. } => unreachable!(),
        }
    }
}
