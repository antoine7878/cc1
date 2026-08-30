use crate::ast::{
    DeclaratorArena, EnumArena, ExpressionArena, StatementArena, StringArena, StructArena, UnionArena, VariantArena,
};

/// Arenas for AST nodes, populated by the lexer and parser and never mutated
/// afterwards. Arenas produced by later phases live elsewhere (e.g. `Sema`).
#[derive(Debug, Default)]
pub struct AstArenas {
    pub names: StringArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
    pub statements: StatementArena,
}
