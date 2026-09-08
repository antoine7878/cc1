#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    PostInc,
    PostDec,
    PreInc,
    PreDec,
    Addr,
    Deref,
    Plus,
    Minus,
    BitNot,
    LogicalNot,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Left,
    Right,
    Greater,
    Lower,
    GreaterEq,
    LowerEq,
    Eq,
    Neq,
    BitAnd,
    BitOr,
    BitXor,
    LogicalAnd,
    LogicalOr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberOp {
    Dot,
    Arrow,
}
