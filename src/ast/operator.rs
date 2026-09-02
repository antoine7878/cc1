use std::fmt::Display;

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

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UnaryOp::PostInc => "post ++",
            UnaryOp::PostDec => "post --",
            UnaryOp::PreInc => "pre ++",
            UnaryOp::PreDec => "pre --",
            UnaryOp::Addr => "Addr",
            UnaryOp::Deref => "Deref",
            UnaryOp::Plus => "Plus",
            UnaryOp::Minus => "Minus",
            UnaryOp::BitNot => "BitNot",
            UnaryOp::LogicalNot => "LogicalNot",
        };
        write!(f, "{}", s)
    }
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

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::Add => "Add",
            BinaryOp::Sub => "Sub",
            BinaryOp::Mul => "Mul",
            BinaryOp::Div => "Div",
            BinaryOp::Mod => "Mod",
            BinaryOp::Left => "Left",
            BinaryOp::Right => "Right",
            BinaryOp::Greater => "Greater",
            BinaryOp::Lower => "Lower",
            BinaryOp::GreaterEq => "GreaterEq",
            BinaryOp::LowerEq => "LowerEq",
            BinaryOp::Eq => "Eq",
            BinaryOp::Neq => "Neq",
            BinaryOp::BitAnd => "BitAnd",
            BinaryOp::BitOr => "BitOr",
            BinaryOp::BitXor => "BitXor",
            BinaryOp::LogicalAnd => "LogicalAnd",
            BinaryOp::LogicalOr => "LogicalOr",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberOp {
    Dot,
    Arrow,
}

impl MemberOp {
    pub fn symbol(&self) -> &str {
        match self {
            MemberOp::Dot => ".",
            MemberOp::Arrow => "->",
        }
    }
}
impl Display for MemberOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemberOp::Dot => "Dot access",
            MemberOp::Arrow => "Ptr access",
        };
        write!(f, "{}", s)
    }
}
