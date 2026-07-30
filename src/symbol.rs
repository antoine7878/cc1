use std::collections::HashMap;

use crate::arena::{Arena, ArenaId};
use crate::context::Arenas;
use crate::types::TypeId;

pub type NameId = ArenaId<String>;
pub type NameArena = Arena<NameId, String>;

impl NameArena {
    pub fn add(&mut self, name: String) -> NameId {
        self.alloc(name)
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_ordinary(name, arenas))
    }

    pub fn get_tag(&self, kind: TagKind, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_tag(kind, name, arenas))
    }

    pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_label(name, arenas))
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    ordinaries: HashMap<NameId, SymbolData>,
    tags: HashMap<(TagKind, NameId), SymbolData>,
    labels: HashMap<NameId, SymbolData>,
}

impl Scope {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.ordinaries.get(arenas.names.canonical.get(name)?)
    }

    pub fn get_tag(&self, kind: TagKind, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        let id = *arenas.names.canonical.get(name)?;
        self.tags.get(&(kind, id))
    }

    pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.labels.get(arenas.names.canonical.get(name)?)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SymbolData {
    pub name: NameId,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub storage: StorageKind,
    pub linkage: LinkageKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SymbolKind {
    Ordinary,
    Tag,
    Label,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum StorageKind {
    Auto,
    Static,
    Extern,
    Register,
    ThreadLocal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkageKind {
    External,
    Internal,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TagKind {
    Struct,
    Union,
    Enum,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Label {
    pub name: NameId,
    pub defined: bool,
}
