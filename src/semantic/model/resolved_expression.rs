use crate::semantic::{ImplicitCast, QualifiedType};

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    RValue,
    LValue,
}

#[derive(Clone, Debug)]
pub struct ResolvedExpression {
    pub ty: QualifiedType,
    pub kind: ExpressionKind,
    pub casts: Vec<ImplicitCast>,
}

impl ResolvedExpression {
    pub fn value_ty(&self) -> Option<QualifiedType> {
        self.casts.last().map(|c| c.to).or(Some(self.ty))
    }

    pub fn new(ty: QualifiedType, kind: ExpressionKind) -> Self {
        Self {
            ty,
            kind,
            casts: Vec::new(),
        }
    }
}
