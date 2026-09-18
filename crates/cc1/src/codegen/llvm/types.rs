use std::fmt;

use crate::ast::{ConstValue, Tag};
use crate::context::ctx;
use crate::semantic::{ParamTypes, QualifiedType, ResolvedType, ResolvedTypeId, TagDefId, sema};

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
    Array(usize, ResolvedTypeId),
    Tag(TagDefId),
    Function(ResolvedTypeId),
}

impl LlvmType {
    pub fn ptr_size() -> Self {
        Self::integer(ctx().target.pointer.size)
    }

    pub fn ptr() -> Self {
        LlvmType::Ptr
    }

    pub fn void() -> Self {
        LlvmType::Void
    }

    pub fn bool() -> Self {
        LlvmType::I1
    }

    pub fn char() -> Self {
        Self::integer(ctx().target.char.size)
    }

    pub fn int() -> Self {
        Self::integer(ctx().target.int.size)
    }

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
            LlvmType::I1 => 0,
            LlvmType::I8 => 1,
            LlvmType::I16 => 2,
            LlvmType::I32 => 4,
            LlvmType::I64 => 8,
            LlvmType::F32 => 4,
            LlvmType::F64 => 8,
            LlvmType::F80 => sema().target.long_double.size,
            LlvmType::Ptr => sema().target.pointer.size,
            LlvmType::Void => 0,
            LlvmType::Array(len, elem) => *len as u32 * elem.llvm().size(),
            LlvmType::Tag(id) => {
                let ty = sema().types.lookup(&ResolvedType::Tag(*id)).expect("unregistered tag type");
                sema().layout(&ty).size
            }
            LlvmType::Function(_) => sema().target.pointer.size,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, LlvmType::Void)
    }
}

impl From<ConstValue> for LlvmType {
    fn from(value: ConstValue) -> Self {
        match value {
            ConstValue::Int(_) | ConstValue::UnsignedInt(_) => LlvmType::integer(ctx().target.int.size),
            ConstValue::Long(_) | ConstValue::UnsignedLong(_) => LlvmType::integer(ctx().target.long.size),
            ConstValue::Float(_) => LlvmType::float(ctx().target.float.size),
            ConstValue::Double(_) => LlvmType::float(ctx().target.double.size),
            ConstValue::LongDouble(_) => LlvmType::float(ctx().target.long_double.size),
        }
    }
}

impl From<&ResolvedTypeId> for LlvmType {
    fn from(value: &ResolvedTypeId) -> Self {
        match value.resolve() {
            ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::UnsignedChar
            | ResolvedType::Short
            | ResolvedType::UnsignedShort
            | ResolvedType::Long
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong
            | ResolvedType::Int => LlvmType::integer(sema().layout(value).size),
            ResolvedType::Float | ResolvedType::Double | ResolvedType::LongDouble => {
                LlvmType::float(sema().layout(value).size)
            }
            ResolvedType::Void => LlvmType::Void,
            ResolvedType::Pointer(_) => LlvmType::Ptr,
            ResolvedType::Function { .. } => LlvmType::Function(*value),
            ResolvedType::Array { elem, len } => LlvmType::Array(len.unwrap_or(0), elem.id),
            ResolvedType::Tag(id) => LlvmType::Tag(*id),
        }
    }
}

impl fmt::Display for LlvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlvmType::I1 => write!(f, "i1"),
            LlvmType::I8 => write!(f, "i8"),
            LlvmType::I16 => write!(f, "i16"),
            LlvmType::I32 => write!(f, "i32"),
            LlvmType::I64 => write!(f, "i64"),
            LlvmType::F32 => write!(f, "float"),
            LlvmType::F64 => write!(f, "double"),
            LlvmType::F80 => write!(f, "x86_fp80"),
            LlvmType::Ptr => write!(f, "ptr"),
            LlvmType::Void => write!(f, "void"),
            LlvmType::Array(len, t) => write!(f, "[{len} x {}]", t.llvm()),
            LlvmType::Tag(id) => tag_name(f, *id),
            LlvmType::Function(id) => function_type(f, *id),
        }
    }
}

fn function_type(f: &mut fmt::Formatter<'_>, id: ResolvedTypeId) -> fmt::Result {
    let ResolvedType::Function { ret, params } = id.resolve() else {
        unreachable!("LlvmType::Function on a non-function")
    };
    write!(f, "{} (", ret.llvm())?;
    match params {
        ParamTypes::Unspecified => write!(f, "...")?,
        ParamTypes::Prototype { params, is_variadic } => {
            for (i, p) in params.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", p.llvm())?;
            }
            if *is_variadic {
                write!(f, "{}", if params.is_empty() { "..." } else { ", ..." })?;
            }
        }
    }
    write!(f, ")")
}

fn tag_name(f: &mut fmt::Formatter<'_>, id: TagDefId) -> fmt::Result {
    let def = id.resolve();
    let kind = match def.kind {
        Tag::Struct => "struct",
        Tag::Union => "union",
        Tag::Enum => return write!(f, "{}", LlvmType::int()),
    };
    let Some(name) = def.name else {
        return write!(f, "%{kind}.anon.{}", usize::from(id));
    };
    let shadowed = sema().tags.iter().filter(|t| t.name.is_some_and(|n| n.id == name.id)).count() > 1;
    match shadowed {
        true => write!(f, "%{kind}.{}.{}", name.id.resolve(), usize::from(id)),
        false => write!(f, "%{kind}.{}", name.id.resolve()),
    }
}
