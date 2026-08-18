use crate::ast::{DeclaratorNode, Name, Storage};
use crate::define_arena;
use crate::semantic::QualifiedType;

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

impl SymbolArena {
    pub fn add(&mut self, name: Name, ty: QualifiedType, storage: Storage, declarator: DeclaratorNode) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            declarator,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Symbol {
    pub name: Name,
    pub ty: QualifiedType,
    pub storage: Storage,
    pub declarator: DeclaratorNode,
}
