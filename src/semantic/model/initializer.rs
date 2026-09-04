use crate::{
    ast::{ExpressionId, StringId, Value},
    define_arena,
    semantic::{Sema, SymbolId},
};

define_arena!(Initilizer, InitilizerArena, InitilizerId, Sema, sema, inits);
#[derive(Clone, Debug)]
pub enum Initilizer {
    Zero,
    Value(Value),
    Address { sym: SymbolId, offset: u32 },
    String(StringId),
    List(Vec<Initilizer>),
    Expr(ExpressionId),
}
