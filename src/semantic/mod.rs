pub mod declarations;
pub mod diagnosis;
pub mod resolved_type;
pub mod symbol;

pub use diagnosis::{Diag, Diagnosis};
pub use resolved_type::{ResolvedType, TypeSpecifierCounter};
pub use symbol::{Analyzer, ScopeType, SymbolArena};
