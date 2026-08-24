use std::collections::HashMap;

use crate::ast::StringId;
use crate::semantic::{SymbolId, SymbolKind, TagDefId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<StringId, TagDefId>,
    labels: HashMap<StringId, SymbolId>,
    ordinaries: HashMap<StringId, SymbolId>,
    kind: ScopeKind,
}

impl Scope {
    fn new(kind: ScopeKind) -> Self {
        Self {
            tags: HashMap::new(),
            labels: HashMap::new(),
            ordinaries: HashMap::new(),
            kind,
        }
    }
}

#[derive(Default, Debug)]
pub struct Scopes(Vec<Scope>);

impl Scopes {
    pub fn push(&mut self, kind: ScopeKind) {
        self.0.push(Scope::new(kind));
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn last(&self) -> &Scope {
        self.0.last().unwrap()
    }

    fn last_mut(&mut self) -> &mut Scope {
        self.0.last_mut().unwrap()
    }

    fn function(&self) -> &Scope {
        self.0.iter().find(|s| s.kind == ScopeKind::Function).unwrap()
    }

    fn function_mut(&mut self) -> &mut Scope {
        self.0.iter_mut().find(|s| s.kind == ScopeKind::Function).unwrap()
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

    pub fn lookup_label(&self, name: StringId) -> Option<SymbolId> {
        self.0.iter().rev().find_map(|s| s.labels.get(&name)).copied()
    }

    pub fn lookup_tag(&self, name: StringId, current_only: bool) -> Option<TagDefId> {
        if current_only {
            return self.last().tags.get(&name).copied();
        }
        self.0.iter().rev().find_map(|s| s.tags.get(&name)).copied()
    }

    pub fn current(&self, kind: SymbolKind, name: StringId) -> Option<SymbolId> {
        match kind {
            SymbolKind::Label => self.function().labels.get(&name).copied(),
            SymbolKind::Typedef
            | SymbolKind::Variable
            | SymbolKind::Parameter
            | SymbolKind::Function
            | SymbolKind::Variant => self.last().ordinaries.get(&name).copied(),
            _ => unimplemented!(),
        }
    }

    pub fn insert(&mut self, kind: SymbolKind, name: StringId, id: SymbolId) {
        match kind {
            SymbolKind::Label => self.function_mut().labels.insert(name, id),
            SymbolKind::Typedef
            | SymbolKind::Variable
            | SymbolKind::Parameter
            | SymbolKind::Function
            | SymbolKind::Variant => self.last_mut().ordinaries.insert(name, id),
            _ => unimplemented!(),
        };
    }

    pub fn insert_tag(&mut self, name: StringId, id: TagDefId) {
        self.last_mut().tags.insert(name, id);
    }
}
