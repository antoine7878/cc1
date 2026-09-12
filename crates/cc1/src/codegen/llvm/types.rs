use std::fmt;

use crate::{
    ast::ConstValue,
    context::ctx,
    semantic::{QualifiedType, ResolvedType, ResolvedTypeId, sema},
};

impl ResolvedTypeId {
    pub fn llvm(&self) -> LlvmType {
        LlvmType::from(self)
    }
}

impl QualifiedType {
    pub fn llvm(&self) -> LlvmType {
        self.id.llvm()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LlvmType {
    First(LlvmFirstType),
    Array(usize, LlvmFirstType),
}

impl LlvmType {
    pub fn ptr_size() -> Self {
        LlvmType::First(LlvmFirstType::integer(ctx().target.pointer.size))
    }

    pub fn ptr() -> Self {
        LlvmType::First(LlvmFirstType::Ptr)
    }

    pub fn void() -> Self {
        LlvmType::First(LlvmFirstType::Void)
    }

    pub fn bool() -> Self {
        LlvmType::First(LlvmFirstType::I1)
    }

    pub fn char() -> Self {
        LlvmType::First(LlvmFirstType::integer(ctx().target.char.size))
    }

    pub fn int() -> Self {
        LlvmType::First(LlvmFirstType::integer(ctx().target.int.size))
    }

    pub fn size(&self) -> u32 {
        match self {
            LlvmType::First(f) => f.size(),
            LlvmType::Array(len, elem) => *len as u32 * elem.size(),
        }
    }
}

impl From<&ResolvedTypeId> for LlvmType {
    fn from(value: &ResolvedTypeId) -> Self {
        match value.resolve() {
            ResolvedType::Array { elem, len } => LlvmType::Array(len.unwrap(), LlvmFirstType::from(&elem.id)),
            _ => LlvmType::First(LlvmFirstType::from(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LlvmFirstType {
    I1,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    F80,
    Void,
    Ptr,
}

impl LlvmFirstType {
    fn integer(i: u32) -> Self {
        match i {
            1 => Self::I8,
            2 => Self::I16,
            4 => Self::I32,
            8 => Self::I64,
            _ => unimplemented!(),
        }
    }

    fn float(i: u32) -> Self {
        match i {
            4 => Self::F32,
            8 => Self::F64,
            12 | 16 => Self::F80,
            _ => unimplemented!(),
        }
    }

    pub fn size(&self) -> u32 {
        match self {
            LlvmFirstType::I1 => 0,
            LlvmFirstType::I8 => 1,
            LlvmFirstType::I16 => 2,
            LlvmFirstType::I32 => 4,
            LlvmFirstType::I64 => 8,
            LlvmFirstType::F32 => 4,
            LlvmFirstType::F64 => 8,
            LlvmFirstType::F80 => sema().target.long_double.size,
            LlvmFirstType::Ptr => sema().target.pointer.size,
            LlvmFirstType::Void => 0,
        }
    }
}

impl From<ConstValue> for LlvmType {
    fn from(value: ConstValue) -> Self {
        LlvmType::First(value.into())
    }
}

impl From<ConstValue> for LlvmFirstType {
    fn from(value: ConstValue) -> Self {
        match value {
            ConstValue::Int(_) | ConstValue::UnsignedInt(_) => LlvmFirstType::integer(ctx().target.int.size),
            ConstValue::Long(_) | ConstValue::UnsignedLong(_) => LlvmFirstType::integer(ctx().target.long.size),
            ConstValue::Float(_) => LlvmFirstType::integer(ctx().target.float.size),
            ConstValue::Double(_) => LlvmFirstType::integer(ctx().target.float.size),
            ConstValue::LongDouble(_) => LlvmFirstType::integer(ctx().target.float.size),
        }
    }
}

impl From<&ResolvedTypeId> for LlvmFirstType {
    fn from(ty: &ResolvedTypeId) -> Self {
        match ty.resolve() {
            ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::UnsignedChar
            | ResolvedType::Short
            | ResolvedType::UnsignedShort
            | ResolvedType::Long
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong
            | ResolvedType::Int => LlvmFirstType::integer(sema().layout(ty).size),
            ResolvedType::Float | ResolvedType::Double | ResolvedType::LongDouble => {
                LlvmFirstType::float(sema().layout(ty).size)
            }
            ResolvedType::Void => LlvmFirstType::Void,
            ResolvedType::Pointer(_) => LlvmFirstType::Ptr,
            // ResolvedType::Function { .. } => todo!(),
            // ResolvedType::Tag { .. } => todo!(),
            _ => todo!(),
        }
    }
}

impl fmt::Display for LlvmFirstType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlvmFirstType::I1 => write!(f, "i1"),
            LlvmFirstType::I8 => write!(f, "i8"),
            LlvmFirstType::I16 => write!(f, "i16"),
            LlvmFirstType::I32 => write!(f, "i32"),
            LlvmFirstType::I64 => write!(f, "i64"),
            LlvmFirstType::F32 => write!(f, "f32"),
            LlvmFirstType::F64 => write!(f, "f64"),
            LlvmFirstType::F80 => write!(f, "x86_fp60"),
            LlvmFirstType::Ptr => write!(f, "ptr"),
            LlvmFirstType::Void => write!(f, "void"),
        }
    }
}
impl fmt::Display for LlvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlvmType::First(t) => write!(f, "{t}"),
            LlvmType::Array(len, t) => write!(f, "[{len} x {t}]"),
        }
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct LlvmType {
//     pub ty: &'static ResolvedType,
// }
//
// impl LlvmType {
//     pub fn bool() -> Self {
//         Self { ty: &ResolvedType::Bool }
//     }
//
//     pub fn char() -> Self {
//         Self { ty: &ResolvedType::Char }
//     }
//
//     pub fn int() -> Self {
//         Self { ty: &ResolvedType::Int }
//     }
// }
