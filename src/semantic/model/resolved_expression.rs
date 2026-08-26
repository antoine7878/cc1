use crate::ast::Value;
use crate::semantic::{QualifiedType, SymbolId};

#[derive(Clone, Copy, Debug)]
pub enum ValueKind {
    RValue,
    LValue,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedExpression {
    pub sym: Option<SymbolId>,
    pub ty: Option<QualifiedType>,
    pub const_value: Option<Option<Value>>,
    pub kind: Option<ValueKind>,
}
