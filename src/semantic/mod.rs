pub mod diagnosis;
mod display;
pub mod resolution;
pub mod resolved_type;
pub mod scope;
pub mod symbol;
pub mod tag_def;

pub mod constrain;

pub use diagnosis::{Diag, DiagCollector, Diagnosis, DiagnosisNode, Severity};
pub use resolution::Analyzer;
pub use resolved_type::{QualifiedType, ResolvedType, ResolvedTypeArena, ResolvedTypeId, TypeSpecifierCounter};
pub use scope::{ScopeKind, Scopes};
pub use symbol::{Symbol, SymbolArena, SymbolId, SymbolKind};
pub use tag_def::{TagDef, TagDefArena, TagDefId};
