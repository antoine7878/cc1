use std::fmt;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::semantic::{ExpressionKind, QualifiedType, ScopeKind, Sema};

define_arena!(Symbol, SymbolArena, SymbolId, crate::semantic::Sema, sema, symbols);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Linkage {
    None,
    External,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duration {
    None,
    Static,
    Automatic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Definition {
    Tentative,
    Definition,
    Declaration,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: Name,
    pub ty: Option<QualifiedType>,
    pub storage: Option<Storage>,
    pub kind: SymbolKind,
    pub value: Option<i32>,
    pub linkage: Linkage,
    pub duration: Duration,
    pub definition: Definition,
    pub is_init: bool,
    pub used: bool,
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
            linkage: Linkage::None,
            definition: Definition::Definition,
            duration: Duration::None,
            used: false,
        }
    }

    pub fn linkage_of(scope: ScopeKind, storage: Option<Storage>, kind: SymbolKind, prior: Option<Linkage>) -> Linkage {
        if !matches!(kind, SymbolKind::Variable | SymbolKind::Function) {
            return Linkage::None;
        }
        if storage == Some(Storage::Extern) {
            return prior.unwrap_or(Linkage::External);
        }
        if kind == SymbolKind::Function {
            return match storage {
                Some(Storage::Static) => Linkage::Internal,
                _ => prior.unwrap_or(Linkage::External),
            };
        }
        if scope == ScopeKind::Block || scope == ScopeKind::Function {
            return Linkage::None;
        }
        if storage == Some(Storage::Static) {
            return Linkage::Internal;
        }
        Linkage::External
    }

    pub fn duration_of(scope: ScopeKind, storage: Option<Storage>, kind: SymbolKind) -> Duration {
        if !matches!(kind, SymbolKind::Variable | SymbolKind::Parameter) {
            return Duration::None;
        }
        if scope == ScopeKind::File || storage == Some(Storage::Static) || storage == Some(Storage::Extern) {
            Duration::Static
        } else {
            Duration::Automatic
        }
    }

    pub fn definition_of(scope: ScopeKind, storage: Option<Storage>, has_initializer: bool, kind: SymbolKind) -> Definition {
        if kind == SymbolKind::Function {
            return Definition::Declaration;
        }
        if has_initializer {
            return Definition::Definition;
        }
        if scope == ScopeKind::File {
            return match storage {
                None | Some(Storage::Static) => Definition::Tentative,
                _ => Definition::Declaration,
            };
        }
        match storage {
            Some(Storage::Extern) => Definition::Declaration,
            _ => Definition::Definition,
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

    pub fn function(name: Name, ty: QualifiedType, storage: Storage) -> Self {
        Self::new(name, Some(ty), Some(storage), SymbolKind::Function, true)
    }

    pub fn parameter(name: Name, ty: QualifiedType, storage: Storage) -> Self {
        Self {
            duration: Duration::Automatic,
            ..Self::new(name, Some(ty), Some(storage), SymbolKind::Parameter, false)
        }
    }

    pub fn label(name: Name, is_init: bool) -> Self {
        Self::new(name, None, None, SymbolKind::Label, is_init)
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        self.name.id == other.name.id
            && self.kind == other.kind
            && (self.ty == other.ty || Option::zip(self.ty, other.ty).is_some_and(|(a, b)| a.is_compatible(sema, &b)))
    }

    pub fn expression_kind(&self) -> ExpressionKind {
        match self.kind {
            SymbolKind::Function | SymbolKind::Variant => ExpressionKind::RValue,
            _ => ExpressionKind::LValue,
        }
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

impl fmt::Display for Linkage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Linkage::None => write!(f, "none"),
            Linkage::External => write!(f, "external"),
            Linkage::Internal => write!(f, "internal"),
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Duration::None => write!(f, "none"),
            Duration::Static => write!(f, "static"),
            Duration::Automatic => write!(f, "automatic"),
        }
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Definition::Declaration => write!(f, "declaration"),
            Definition::Tentative => write!(f, "tentative"),
            Definition::Definition => write!(f, "definition"),
        }
    }
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
