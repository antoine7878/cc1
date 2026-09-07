use std::collections::HashMap;

use crate::ast::StringId;
use crate::semantic::{SymbolId, TagDefId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
pub struct SymbolScope {
    tags: HashMap<StringId, TagDefId>,
    ordinaries: HashMap<StringId, SymbolId>,
    pub kind: ScopeKind,
}

impl SymbolScope {
    fn new(kind: ScopeKind) -> Self {
        Self {
            tags: HashMap::new(),
            ordinaries: HashMap::new(),
            kind,
        }
    }
}

#[derive(Default, Debug)]
pub struct SymbolScopes(Vec<SymbolScope>);

impl SymbolScopes {
    pub fn push(&mut self, kind: ScopeKind) {
        self.0.push(SymbolScope::new(kind));
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn last(&self) -> &SymbolScope {
        self.0.last().unwrap()
    }

    fn last_mut(&mut self) -> &mut SymbolScope {
        self.0.last_mut().unwrap()
    }

    pub fn kind(&self) -> ScopeKind {
        self.last().kind
    }

    pub fn set_kind(&mut self, kind: ScopeKind) {
        self.last_mut().kind = kind;
    }

    pub fn lookup_ordinary(&self, name: StringId) -> Option<SymbolId> {
        self.0.iter().rev().find_map(|s| s.ordinaries.get(&name)).copied()
    }

    pub fn lookup_tag(&self, name: StringId, current_only: bool) -> Option<TagDefId> {
        if current_only {
            return self.last().tags.get(&name).copied();
        }
        self.0.iter().rev().find_map(|s| s.tags.get(&name)).copied()
    }

    pub fn current(&self, name: StringId) -> Option<SymbolId> {
        self.last().ordinaries.get(&name).copied()
    }

    pub fn insert(&mut self, name: StringId, id: SymbolId) {
        self.last_mut().ordinaries.insert(name, id);
    }

    pub fn insert_tag(&mut self, name: StringId, id: TagDefId) {
        self.last_mut().tags.insert(name, id);
    }
}
