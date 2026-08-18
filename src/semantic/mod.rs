pub mod declarations;
pub mod diagnosis;
pub mod resolution;
pub mod resolved_type;
pub mod symbol;

pub use diagnosis::{Diag, Diagnosis};
pub use resolution::{Analyzer, ScopeType};
pub use resolved_type::{ResolvedType, TypeSpecifierCounter};
pub use symbol::{SymbolArena, SymbolId};
