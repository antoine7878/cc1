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

impl BinaryOp {
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Mod
                | BinaryOp::Left
                | BinaryOp::Right
                | BinaryOp::BitAnd
                | BinaryOp::BitOr
                | BinaryOp::BitXor
        )
    }

    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOp::LogicalAnd | BinaryOp::LogicalOr)
    }

    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Greater
                | BinaryOp::Lower
                | BinaryOp::GreaterEq
                | BinaryOp::LowerEq
                | BinaryOp::Eq
                | BinaryOp::Neq
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberOp {
    Dot,
    Arrow,
}
