use std::fmt;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::semantic::{ExpressionKind, QualifiedType, Sema};

define_arena!(Symbol, SymbolArena, SymbolId, crate::semantic::Sema, symbols);

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: Name,
    pub ty: Option<QualifiedType>,
    pub storage: Option<Storage>,
    pub kind: SymbolKind,
    pub value: Option<i32>,
    pub is_init: bool,
}

impl Symbol {
    pub fn new(
        name: Name,
        ty: Option<QualifiedType>,
        storage: Option<Storage>,
        kind: SymbolKind,
        is_init: bool,
    ) -> Self {
        Self {
            name,
            ty,
            storage,
            kind,
            value: None,
            is_init,
        }
    }

    fn with_value(name: Name, ty: QualifiedType, value: Option<i32>, kind: SymbolKind) -> Self {
        Self {
            value,
            ..Self::new(name, Some(ty), None, kind, true)
        }
    }

    pub fn member(name: Name, ty: QualifiedType, value: Option<i32>) -> Self {
        Self::with_value(name, ty, value, SymbolKind::Member)
    }

    pub fn variant(name: Name, ty: QualifiedType, value: i32) -> Self {
        Self::with_value(name, ty, Some(value), SymbolKind::Variant)
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        self.name.id == other.name.id
            && self.kind == other.kind
            && (self.ty == other.ty
                || Option::zip(self.ty, other.ty)
                    .map(|(a, b)| a.is_compatible(sema, &b))
                    .unwrap_or(false))
    }

    pub fn expression_kind(&self) -> ExpressionKind {
        match self.kind {
            SymbolKind::Function | SymbolKind::Variant => ExpressionKind::RValue,
            _ => ExpressionKind::LValue,
        }
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
        self.alloc(Symbol::new(name, ty, storage, kind, is_init))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
