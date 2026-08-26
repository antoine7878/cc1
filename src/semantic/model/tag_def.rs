use crate::ast::{Name, Tag};
use crate::define_arena;
use crate::semantic::{SymbolId, SymbolKind};

define_arena!(TagDef, TagDefArena, TagDefId, tags);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagDef {
    pub kind: Tag,
    pub name: Option<Name>,
    pub members: Vec<SymbolId>,
    pub is_complete: bool,
}

impl Tag {
    pub fn symbol_kind(self) -> SymbolKind {
        match self {
            Tag::Struct => SymbolKind::Struct,
            Tag::Union => SymbolKind::Union,
            Tag::Enum => SymbolKind::Enum,
        }
    }
}

impl TagDef {
    pub fn kind(&self) -> SymbolKind {
        self.kind.symbol_kind()
    }
}

impl TagDefArena {
    pub fn declare(&mut self, kind: Tag, name: Option<Name>) -> TagDefId {
        self.alloc(TagDef {
            kind,
            name,
            members: Vec::new(),
            is_complete: false,
        })
    }

    pub fn complete(&mut self, id: TagDefId, members: Vec<SymbolId>) {
        let def = self.get_mut(id);
        def.members = members;
        def.is_complete = true;
    }
}
