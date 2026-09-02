use crate::ast::{Name, Tag};
use crate::define_arena;
use crate::semantic::{Sema, SymbolId, SymbolKind};

define_arena!(TagDef, TagDefArena, TagDefId, crate::semantic::Sema, sema, tags);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagDef {
    pub kind: Tag,
    pub name: Option<Name>,
    pub members: Vec<Member>,
    pub is_complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Member {
    Symbol(SymbolId),
    Bitfield(i32),
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

    pub fn is_enum(&self) -> bool {
        matches!(self.kind, Tag::Enum)
    }

    pub fn get_member(&self, sema: &Sema, name: &Name) -> Option<SymbolId> {
        self.members.iter().find_map(|&member| {
            let Member::Symbol(sym_id) = member else { return None };
            (sema.symbols.get(sym_id).name.id == name.id).then_some(sym_id)
        })
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

    pub fn complete(&mut self, id: TagDefId, members: Vec<Member>) {
        let def = self.get_mut(id);
        def.members = members;
        def.is_complete = true;
    }
}
