use std::fmt;

use crate::context::Context;
use crate::semantic::{QualifiedType, ResolvedType};

impl ResolvedType {
    pub fn llvm<'a>(&'a self, ctx: &'a Context) -> TyName<'a> {
        TyName::new(self, ctx)
    }
}

impl QualifiedType {
    pub fn llvm<'a>(&'a self, ctx: &'a Context) -> TyName<'a> {
        self.id.resolve().llvm(ctx)
    }
}

pub struct TyName<'a> {
    ty: &'a ResolvedType,
    ctx: &'a Context,
}
impl<'a> TyName<'a> {
    fn new(ty: &'a ResolvedType, ctx: &'a Context) -> Self {
        Self { ty, ctx }
    }
}

impl<'a> fmt::Display for TyName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::UnsignedChar
            | ResolvedType::Short
            | ResolvedType::UnsignedShort
            | ResolvedType::Long
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong
            | ResolvedType::Int => write!(f, "i{}", self.ctx.target.layout(self.ty).unwrap().size * 8),
            ResolvedType::Void => write!(f, "void"),
            ResolvedType::Float => write!(f, "{}", self.ctx.target.float.llvm()),
            ResolvedType::Double => write!(f, "{}", self.ctx.target.double.llvm()),
            ResolvedType::LongDouble => write!(f, "{}", self.ctx.target.long_double.llvm()),
            ResolvedType::Function { .. } => todo!(),
            ResolvedType::Tag { .. } => todo!(),
            ResolvedType::Array { .. } => todo!(),
            ResolvedType::Pointer(_) => todo!(),
        }
    }
}
