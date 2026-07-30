use crate::arena::{Arena, ArenaId};
use crate::symbol::NameId;
use crate::types::TypeId;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Field {
    pub name: NameId,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Struct {
    pub name: Option<NameId>,
    pub fields: Vec<Field>,
    pub complete: bool,
}
pub type StructId = ArenaId<Struct>;
pub type StructArena = Arena<StructId, Struct>;

impl StructArena {
    pub fn add(&mut self, name: Option<NameId>, fields: Vec<Field>, complete: bool) -> StructId {
        self.alloc(Struct { name, fields, complete })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Union {
    pub name: Option<NameId>,
    pub fields: Vec<Field>,
    pub complete: bool,
}
pub type UnionId = ArenaId<Union>;
pub type UnionArena = Arena<UnionId, Union>;

impl UnionArena {
    pub fn add(&mut self, name: Option<NameId>, fields: Vec<Field>, complete: bool) -> UnionId {
        self.alloc(Union { name, fields, complete })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Enum {
    pub name: Option<NameId>,
    pub variants: Vec<VariantId>,
    pub complete: bool,
}
pub type EnumId = ArenaId<Enum>;
pub type EnumArena = Arena<EnumId, Enum>;

impl EnumArena {
    pub fn add(&mut self, name: Option<NameId>, variants: Vec<VariantId>, complete: bool) -> EnumId {
        self.alloc(Enum {
            name,
            variants,
            complete,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Variant {
    pub name: NameId,
    pub value: i64,
}
pub type VariantId = ArenaId<Variant>;
pub type VariantArena = Arena<VariantId, Variant>;

impl VariantArena {
    pub fn add(&mut self, name: NameId, value: i64) -> VariantId {
        self.alloc(Variant { name, value })
    }
}
