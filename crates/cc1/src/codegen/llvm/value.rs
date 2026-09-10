use std::fmt::{self, Display, Formatter};

use crate::ast::ConstValue;

#[derive(Debug, Clone, Copy)]
pub enum LlvmValue {
    SSA(usize),
    Constant(ConstValue),
    StringLiteral(usize),
    Bool(bool),
    Lhs(usize),
    Rhs(usize),
}

impl LlvmValue {
    pub fn fork(i: usize) -> (Self, Self) {
        (Self::Lhs(i), Self::Rhs(i))
    }
}

impl LlvmValue {
    pub fn zero_cst() -> Self {
        LlvmValue::Constant(ConstValue::zero())
    }

    pub fn one_cst() -> Self {
        LlvmValue::Constant(ConstValue::one())
    }

    pub fn minus_one_cst() -> Self {
        LlvmValue::Constant(ConstValue::minus_one())
    }
}

impl Display for LlvmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmValue::SSA(id) => write!(f, "%{id}"),
            LlvmValue::Constant(value) => write!(f, "{value}"),
            LlvmValue::Bool(b) => write!(f, "{b}"),
            LlvmValue::Lhs(s) => write!(f, "%lhs.l.{s}"),
            LlvmValue::Rhs(s) => write!(f, "%rhs.l.{s}"),
            LlvmValue::StringLiteral(i) => write!(f, "@.str.{i}"),
        }
    }
}

impl From<bool> for LlvmValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
