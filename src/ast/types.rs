use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;

use crate::arena::{Arena, ArenaId};
use crate::ast::{EnumId, StructId, UnionId};
use crate::define_arena;
use crate::parser::Span;

define_arena!(Type, TypeArena, TypeId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TypeNode {
    span: Span,
    id: TypeId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Type {
    Void,
    Char,
    Short,
    Int,
    Long,
    Signed,
    Unsigned,

    Float,
    Double,
    LongDouble,
    Pointer(TypeNode),

    Array {
        element: TypeNode,
        length: Option<u64>,
    },

    Function {
        return_type: TypeNode,
        params: Vec<TypeNode>,
        variadic: bool,
    },
    Struct(StructId),
    Union(UnionId),
    Enum(EnumId),
    Qualified {
        base: TypeNode,
        qualifiers: BTreeSet<Qualifier>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Qualifier {
    Const,
    Volatile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TagKind {
    Struct,
    Union,
    Enum,
}

impl TypeArena {
    fn add(id: TypeId, span: Span) -> TypeNode {
        TypeNode { id, span }
    }
    pub fn void(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Void), span)
    }

    pub fn char(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Char), span)
    }

    pub fn short(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Short), span)
    }
    pub fn int(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Int), span)
    }

    pub fn long(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Long), span)
    }

    pub fn signed(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Signed), span)
    }

    pub fn unsigned(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Unsigned), span)
    }

    pub fn float(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Float), span)
    }

    pub fn double(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Double), span)
    }

    pub fn longdouble(&mut self, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::LongDouble), span)
    }

    pub fn pointer(&mut self, base: TypeNode, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Pointer(base)), span)
    }

    pub fn array(&mut self, element: TypeNode, length: Option<u64>, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Array { element, length }), span)
    }

    pub fn function(&mut self, return_type: TypeNode, params: Vec<TypeNode>, variadic: bool, span: Span) -> TypeNode {
        Self::add(
            self.alloc(Type::Function {
                return_type,
                params,
                variadic,
            }),
            span,
        )
    }

    pub fn struct_(&mut self, base: StructId, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Struct(base)), span)
    }

    pub fn union_(&mut self, base: UnionId, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Union(base)), span)
    }

    pub fn enum_(&mut self, base: EnumId, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Enum(base)), span)
    }

    pub fn qualified(&mut self, base: TypeNode, qualifiers: BTreeSet<Qualifier>, span: Span) -> TypeNode {
        Self::add(self.alloc(Type::Qualified { base, qualifiers }), span)
    }
}
