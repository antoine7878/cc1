use std::fmt;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
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
    Enumerator,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Parameter => write!(f, "parameter"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Union => write!(f, "union"),
            SymbolKind::Member => write!(f, "member"),
            SymbolKind::Label => write!(f, "label"),
            SymbolKind::Typedef => write!(f, "typedef"),
            SymbolKind::Enumerator => write!(f, "enumerator"),
        }
    }
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
