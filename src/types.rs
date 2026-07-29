use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;

use crate::arena::{Arena, ArenaId};
use crate::tag::{EnumId, StructId, UnionId};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Type {
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,

    Float,
    Double,
    LongDouble,
    Pointer(TypeId),

    Array {
        element: TypeId,
        length: Option<u64>,
    },

    Function {
        return_type: TypeId,
        params: Vec<TypeId>,
        variadic: bool,
    },
    Struct(StructId),
    Union(UnionId),
    Enum(EnumId),
    Qualified {
        base: TypeId,
        qualifiers: BTreeSet<Qualifiers>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Qualifiers {
    Const,
    Volatile,
}

pub type TypeId = ArenaId<Type>;
pub type TypeArena = Arena<TypeId, Type>;

impl TypeArena {
    pub fn void(&mut self) -> TypeId {
        self.alloc(Type::Void)
    }

    pub fn char(&mut self) -> TypeId {
        self.alloc(Type::Char)
    }

    pub fn short(&mut self) -> TypeId {
        self.alloc(Type::Short)
    }
    pub fn int(&mut self) -> TypeId {
        self.alloc(Type::Int)
    }

    pub fn long(&mut self) -> TypeId {
        self.alloc(Type::Long)
    }

    pub fn longlong(&mut self) -> TypeId {
        self.alloc(Type::LongLong)
    }

    pub fn float(&mut self) -> TypeId {
        self.alloc(Type::Float)
    }

    pub fn double(&mut self) -> TypeId {
        self.alloc(Type::Double)
    }

    pub fn longdouble(&mut self) -> TypeId {
        self.alloc(Type::LongDouble)
    }

    pub fn pointer(&mut self, base: TypeId) -> TypeId {
        self.alloc(Type::Pointer(base))
    }

    pub fn array(&mut self, element: TypeId, length: Option<u64>) -> TypeId {
        self.alloc(Type::Array { element, length })
    }

    pub fn function(&mut self, return_type: TypeId, params: Vec<TypeId>, variadic: bool) -> TypeId {
        self.alloc(Type::Function {
            return_type,
            params,
            variadic,
        })
    }

    pub fn struct_(&mut self, base: StructId) -> TypeId {
        self.alloc(Type::Struct(base))
    }

    pub fn union_(&mut self, base: UnionId) -> TypeId {
        self.alloc(Type::Union(base))
    }

    pub fn enum_(&mut self, base: EnumId) -> TypeId {
        self.alloc(Type::Enum(base))
    }

    pub fn qualified(&mut self, base: TypeId, qualifiers: BTreeSet<Qualifiers>) -> TypeId {
        self.alloc(Type::Qualified { base, qualifiers })
    }
}
