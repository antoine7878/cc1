use crate::ast::{
    DeclaratorArena, EnumArena, ExpressionArena, StatementArena, StringArena, StringPool, StructArena, UnionArena,
    VariantArena,
};

#[derive(Debug, Default)]
pub struct AstArenas {
    pub names: StringArena,
    pub strings: StringPool,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
    pub statements: StatementArena,
}
