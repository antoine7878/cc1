use crate::arena::{Arena, ArenaId};
use crate::ast::{StringId, TypeId};
use crate::define_arena;

define_arena!(Struct, StructArena, StructId);
define_arena!(Union, UnionArena, UnionId);
define_arena!(Enum, EnumArena, EnumId);
define_arena!(Variant, VariantArena, VariantId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Field {
    pub name: StringId,
    pub ty: TypeId,
}
impl Field {
    pub fn new(name: StringId, ty: TypeId) -> Field {
        Field { name, ty }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Struct {
    pub name: Option<StringId>,
    pub fields: Vec<Field>,
    pub complete: bool,
}

impl StructArena {
    pub fn add(&mut self, name: Option<StringId>, fields: Vec<Field>, complete: bool) -> StructId {
        self.alloc(Struct { name, fields, complete })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Union {
    pub name: Option<StringId>,
    pub fields: Vec<Field>,
    pub complete: bool,
}

impl UnionArena {
    pub fn add(&mut self, name: Option<StringId>, fields: Vec<Field>, complete: bool) -> UnionId {
        self.alloc(Union { name, fields, complete })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Enum {
    pub name: Option<StringId>,
    pub variants: Vec<VariantId>,
    pub complete: bool,
}

impl EnumArena {
    pub fn add(&mut self, name: Option<StringId>, variants: Vec<VariantId>, complete: bool) -> EnumId {
        self.alloc(Enum {
            name,
            variants,
            complete,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Variant {
    pub name: StringId,
    pub value: i64,
}

impl VariantArena {
    pub fn add(&mut self, name: StringId, value: i64) -> VariantId {
        self.alloc(Variant { name, value })
    }
}
