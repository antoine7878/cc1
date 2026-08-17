use crate::{
    ast::{DeclarationNode, StringId, Tag},
    context::Arenas,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

#[derive(Debug, Default)]
pub struct Scope {
    ordinaries: HashMap<StringId, DeclarationNode>,
    tags: HashMap<(Tag, StringId), DeclarationNode>,
    // labels: HashMap<StringId, DeclarationNode>,
}

impl SymbolTable {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
        self.scopes.iter().rev().find_map(|s| s.get_ordinary(name, arenas))
    }

    pub fn get_tag(&self, kind: Tag, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
        self.scopes.iter().rev().find_map(|s| s.get_tag(kind, name, arenas))
    }

    // pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
    //     self.scopes.iter().rev().find_map(|s| s.get_label(name, arenas))
    // }
}

impl Scope {
    pub fn get_ordinary(&self, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
        self.ordinaries.get(arenas.names.canonical.get(name)?)
    }

    pub fn get_tag(&self, kind: Tag, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
        let id = *arenas.names.canonical.get(name)?;
        self.tags.get(&(kind, id))
    }

    // pub fn get_label(&self, name: &str, arenas: &Arenas) -> Option<&DeclarationNode> {
    //     self.labels.get(arenas.names.canonical.get(name)?)
    // }
}
