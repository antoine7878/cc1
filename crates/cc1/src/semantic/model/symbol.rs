use std::fmt;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::semantic::{InitializerId, QualifiedType, ScopeKind, Sema, ValueCategory};

define_arena!(Symbol, SymbolArena, SymbolId);

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
pub enum DefinitionState {
    Declared,
    Tentative,
    Defined,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: Name,
    pub ty: QualifiedType,
    pub storage: Option<Storage>,
    pub kind: SymbolKind,
    pub value: Option<i32>,
    pub linkage: Linkage,
    pub duration: Duration,
    pub definition: DefinitionState,
    pub has_initializer: bool,
    pub used: bool,
    pub initializer: Option<InitializerId>,
}

impl Symbol {
    pub fn new(
        name: Name,
        ty: QualifiedType,
        storage: Option<Storage>,
        kind: SymbolKind,
        has_initializer: bool,
    ) -> Self {
        Self {
            name,
            ty,
            storage,
            kind,
            value: None,
            has_initializer,
            linkage: Linkage::None,
            definition: DefinitionState::Defined,
            duration: Duration::None,
            used: false,
            initializer: None,
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

    pub fn definition_of(
        scope: ScopeKind,
        storage: Option<Storage>,
        has_initializer: bool,
        kind: SymbolKind,
    ) -> DefinitionState {
        if kind == SymbolKind::Function {
            return DefinitionState::Declared;
        }
        if has_initializer {
            return DefinitionState::Defined;
        }
        if scope == ScopeKind::File {
            return match storage {
                None | Some(Storage::Static) => DefinitionState::Tentative,
                _ => DefinitionState::Declared,
            };
        }
        match storage {
            Some(Storage::Extern) => DefinitionState::Declared,
            _ => DefinitionState::Defined,
        }
    }

    fn with_value(name: Name, ty: QualifiedType, value: Option<i32>, kind: SymbolKind) -> Self {
        Self { value, ..Self::new(name, ty, None, kind, true) }
    }

    pub fn member(name: Name, ty: QualifiedType, value: Option<i32>) -> Self {
        Self::with_value(name, ty, value, SymbolKind::Member)
    }

    pub fn enumerator(name: Name, ty: QualifiedType, value: i32) -> Self {
        Self::with_value(name, ty, Some(value), SymbolKind::Enumerator)
    }

    pub fn function(name: Name, ty: QualifiedType, storage: Storage) -> Self {
        Self::new(name, ty, Some(storage), SymbolKind::Function, true)
    }

    pub fn param(name: Name, ty: QualifiedType, storage: Storage) -> Self {
        Self { duration: Duration::Automatic, ..Self::new(name, ty, Some(storage), SymbolKind::Parameter, false) }
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        self.name.id == other.name.id
            && self.kind == other.kind
            && (self.ty == other.ty || self.ty.is_compatible(sema, &other.ty))
    }

    pub fn value_category(&self) -> ValueCategory {
        match self.kind {
            SymbolKind::Function | SymbolKind::Enumerator => ValueCategory::RValue,
            _ => ValueCategory::LValue,
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
    Typedef,
    Enumerator,
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

impl fmt::Display for DefinitionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefinitionState::Declared => write!(f, "declaration"),
            DefinitionState::Tentative => write!(f, "tentative"),
            DefinitionState::Defined => write!(f, "definition"),
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
            SymbolKind::Typedef => write!(f, "typedef"),
            SymbolKind::Enumerator => write!(f, "enumerator"),
        }
    }
}
