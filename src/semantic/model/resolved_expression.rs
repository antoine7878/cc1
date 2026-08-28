use crate::semantic::{ImplicitCast, QualifiedType};

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    RValue,
    LValue,
}

#[derive(Clone, Debug)]
pub struct ResolvedExpression {
    pub ty: Option<QualifiedType>,
    pub kind: ExpressionKind,
    pub casts: Vec<ImplicitCast>,
}

impl ResolvedExpression {
    pub fn value_ty(&self) -> Option<QualifiedType> {
        self.casts.last().map(|c| c.to).or(self.ty)
    }

    pub fn new(ty: Option<QualifiedType>, kind: ExpressionKind) -> Self {
        Self {
            ty,
            kind,
            casts: Vec::new(),
        }
    }
}
