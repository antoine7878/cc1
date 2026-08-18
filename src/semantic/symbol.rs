use crate::ast::{DeclaratorNode, Name, Storage};
use crate::define_arena;
use crate::semantic::ResolvedType;

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

impl SymbolArena {
    pub fn add(
        &mut self,
        name: Name,
        ty: ResolvedType,
        storage: Storage,
        is_const: bool,
        is_volatile: bool,
        declarator: DeclaratorNode,
    ) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            is_const,
            is_volatile,
            declarator,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Symbol {
    pub name: Name,
    pub ty: ResolvedType,
    pub storage: Storage,
    pub declarator: DeclaratorNode,
    pub is_const: bool,
    pub is_volatile: bool,
}
