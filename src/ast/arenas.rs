use crate::ast::{
    DeclaratorArena, EnumArena, ExpressionArena, StatementArena, StringArena, StructArena, UnionArena, VariantArena,
};

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
