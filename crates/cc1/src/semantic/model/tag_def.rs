use crate::ast::{Name, Tag};
use crate::define_arena;
use crate::semantic::{Sema, SymbolId, SymbolKind};

define_arena!(TagDef, TagDefArena, TagDefId, Sema, crate::semantic::sema(), tags);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagDef {
    pub kind: Tag,
    pub name: Option<Name>,
    pub members: Vec<Member>,
    pub is_complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Member {
    pub sym: Option<SymbolId>,
    pub width: Option<i32>,
    pub offset: u32,
    pub bit_offset: u32,
}

impl Member {
    pub fn symbol(sym: SymbolId, width: Option<i32>) -> Self {
        Self {
            sym: Some(sym),
            width,
            offset: 0,
            bit_offset: 0,
        }
    }

    pub fn bitfield(width: i32) -> Self {
        Self {
            sym: None,
            width: Some(width),
            offset: 0,
            bit_offset: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemberRef {
    pub tag: TagDefId,
    pub index: usize,
}

impl MemberRef {
    pub fn member(self, sema: &Sema) -> Member {
        sema.tags.get(self.tag).members[self.index]
    }
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

    pub fn find_member(&self, sema: &Sema, name: &Name) -> Option<(usize, SymbolId)> {
        self.members.iter().enumerate().find_map(|(index, member)| {
            let sym_id = member.sym?;
            (sema.symbols.get(sym_id).name.id == name.id).then_some((index, sym_id))
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
