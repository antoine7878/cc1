use std::fmt;

use crate::context::ctx;
use crate::semantic::{QualifiedType, ResolvedType};

impl ResolvedType {
    pub fn llvm(&'static self) -> TyName {
        TyName { ty: self }
    }
}

impl QualifiedType {
    pub fn llvm(&self) -> TyName {
        self.id.resolve().llvm()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TyName {
    ty: &'static ResolvedType,
}

impl fmt::Display for TyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = &ctx().target;
        match self.ty {
            ResolvedType::Char
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
            ResolvedType::Function { .. } => todo!(),
            ResolvedType::Tag { .. } => todo!(),
            ResolvedType::Array { .. } => todo!(),
            ResolvedType::Pointer(_) => todo!(),
        }
    }
}
