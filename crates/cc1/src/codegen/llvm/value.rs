use crate::ast::Value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum LlvmValue {
    SSA(usize),
    Literal(Value),
}

impl Display for LlvmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlvmValue::SSA(id) => write!(f, "%{id}"),
            LlvmValue::Literal(value) => write!(f, "{value}"),
        }
    }
}
