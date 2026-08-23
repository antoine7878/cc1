use std::fmt;
use std::hash::Hash;

use crate::ast::{ExpressionNode, Name, Storage};
use crate::define_arena;
use crate::semantic::QualifiedType;

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Symbol {
    pub name: Name,
    pub ty: Option<QualifiedType>,
    pub storage: Option<Storage>,
    pub kind: SymbolKind,
    pub bit_width: Option<ExpressionNode>,
    // pub bit_width: Option<u8>,
    // pub value: Option<i64>,
    pub is_complete: bool,
    pub is_init: bool,
}

impl Symbol {
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.name.id == other.name.id
            && self.ty == other.ty
            && self.storage == other.storage
            && self.kind == other.kind
            && self.bit_width == other.bit_width
    }
}

impl SymbolArena {
    pub fn add(
        &mut self,
        name: Name,
        ty: Option<QualifiedType>,
        storage: Option<Storage>,
        kind: SymbolKind,
        is_init: bool,
    ) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            kind,
            bit_width: None,
            is_complete: true,
            is_init,
        })
    }
    pub fn with_size(
        &mut self,
        name: Name,
        ty: Option<QualifiedType>,
        storage: Option<Storage>,
        kind: SymbolKind,
        size: Option<ExpressionNode>,
    ) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            kind,
            bit_width: size,
            is_complete: true,
            is_init: true,
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
    Variant,
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
            SymbolKind::Variant => write!(f, "variant"),
        }
    }
}
