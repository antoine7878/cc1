use crate::ast::{Name, Storage};
use crate::define_arena;
// use crate::parser::Context;
use crate::semantic::QualifiedType;

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

impl SymbolArena {
    pub fn add(&mut self, name: Name, ty: QualifiedType, storage: Storage, kind: SymbolKind) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            kind,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Struct,
    Enum,
    Union,
    Member,
    Label,
    Typedef,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Symbol {
    pub name: Name,
    pub ty: QualifiedType,
    pub storage: Storage,
    pub kind: SymbolKind,
}

// impl Symbol {
//     fn print(&self, ctx: &Context) {
//         println!("{}: ", self.name.id.resolve(ctx));
//     }
// }
