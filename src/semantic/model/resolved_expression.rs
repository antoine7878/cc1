use crate::ast::Value;
use crate::semantic::model::ImplicitCast;
use crate::semantic::{QualifiedType, SymbolId};

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    RValue,
    LValue,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedExpression {
    pub sym: Option<SymbolId>,
    pub ty: Option<QualifiedType>,
    pub const_value: Option<Option<Value>>,
    pub kind: Option<ExpressionKind>,
    pub casts: Vec<ImplicitCast>,
}

impl ResolvedExpression {
    pub fn value_ty(&self) -> Option<QualifiedType> {
        self.casts.last().map(|c| c.to).or(self.ty)
    }
}
