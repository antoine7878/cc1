use std::fmt::{self, Display, Formatter};

use crate::ast::ConstValue;

#[derive(Debug, Clone, Copy)]
pub enum LlvmValue {
    SSA(usize),
    Literal(ConstValue),
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
        LlvmValue::Literal(ConstValue::zero())
    }

    pub fn one_cst() -> Self {
        LlvmValue::Literal(ConstValue::one())
    }

    pub fn minus_one_cst() -> Self {
        LlvmValue::Literal(ConstValue::minus_one())
    }
}

impl Display for LlvmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmValue::SSA(id) => write!(f, "%{id}"),
            LlvmValue::Literal(value) => write!(f, "{value}"),
            LlvmValue::Bool(b) => write!(f, "{b}"),
            LlvmValue::Lhs(s) => write!(f, "%lhs.l.{s}"),
            LlvmValue::Rhs(s) => write!(f, "%rhs.l.{s}"),
        }
    }
}

impl From<bool> for LlvmValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
