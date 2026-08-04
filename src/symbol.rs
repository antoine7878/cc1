use std::collections::HashMap;

use crate::ast::{DeclarationNode, Storage, StringId, Tag, Type};
use crate::context::Arenas;

#[derive(Debug, Default)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes.iter().rev().find_map(|s| s.get_ordinary(name, arenas))
    }

    pub fn get_tag(&self, kind: Tag, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes.iter().rev().find_map(|s| s.get_tag(kind, name, arenas))
    }

    pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.scopes.iter().rev().find_map(|s| s.get_label(name, arenas))
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    ordinaries: HashMap<StringId, DeclarationNode>,
    tags: HashMap<(Tag, StringId), DeclarationNode>,
    labels: HashMap<StringId, DeclarationNode>,
}

impl Scope {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&> {
        self.ordinaries.get(arenas.names.canonical.get(name)?)
    }

    pub fn get_tag(&self, kind: Tag, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        let id = *arenas.names.canonical.get(name)?;
        self.tags.get(&(kind, id))
    }

    pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&SymbolData> {
        self.labels.get(arenas.names.canonical.get(name)?)
    }
}

// #[derive(Debug, Eq, PartialEq)]
// pub struct SymbolData {
//     pub declaration: DeclarationNode,
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
// pub enum SymbolKind {
//     Ordinary,
//     Tag,
//     Label,
// }

// #[derive(Debug, Eq, PartialEq)]
// pub struct Label {
//     pub name: StringId,
//     pub defined: bool,
// }
