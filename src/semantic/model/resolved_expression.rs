use crate::semantic::{ImplicitCast, QualifiedType};

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    RValue,
    LValue,
}

#[derive(Clone, Debug)]
pub struct ResolvedExpression {
    ty: QualifiedType,
    pub kind: ExpressionKind,
    pub casts: Vec<ImplicitCast>,
    pub result_cast: Option<ImplicitCast>,
}

impl ResolvedExpression {
    pub fn casted_ty(&self) -> QualifiedType {
        self.casts.last().map(|c| c.to).unwrap_or(self.ty)
    }

    pub fn new(ty: QualifiedType, kind: ExpressionKind) -> Self {
        Self {
            ty,
            kind,
            casts: Vec::new(),
            result_cast: None,
        }
    }
}
