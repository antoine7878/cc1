pub mod declaration;
pub mod expression;
pub mod externals;
pub mod function;
pub mod resolver;
pub mod scope;
pub mod statement;
pub mod uses;

pub use expression::resolve_expression;
pub use externals::finish_externals;
pub use resolver::SymbolResolver;
pub use uses::mark_uses;
