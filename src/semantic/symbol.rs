use crate::ast::{ExpressionNode, Name, Storage};
use crate::define_arena;
// use crate::parser::Context;
use crate::semantic::QualifiedType;

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

impl SymbolArena {
    pub fn add(&mut self, name: Name, ty: QualifiedType, storage: Option<Storage>, kind: SymbolKind) -> SymbolId {
        self.alloc_fresh(Symbol {
            name,
            ty,
            storage,
            kind,
            size: None,
        })
    }

    pub fn with_size(
        &mut self,
        name: Name,
        ty: QualifiedType,
        storage: Option<Storage>,
        kind: SymbolKind,
        size: Option<ExpressionNode>,
    ) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            kind,
            size,
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
    pub storage: Option<Storage>,
    pub kind: SymbolKind,
    pub size: Option<ExpressionNode>,
}

// impl Symbol {
//     fn print(&self, ctx: &Context) {
//         println!("{}: ", self.name.id.resolve(ctx));
//     }
// }
