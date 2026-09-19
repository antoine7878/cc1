use crate::semantic::{ImplicitCast, QualifiedType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueCategory {
    RValue,
    LValue,
}

#[derive(Clone, Debug)]
pub struct ResolvedExpression {
    pub ty: QualifiedType,
    pub kind: ValueCategory,
    pub casts: Vec<ImplicitCast>,
    pub result_cast: Option<ImplicitCast>,
    pub bit_width: Option<i32>,
}

impl ResolvedExpression {
    pub fn casted_ty(&self) -> QualifiedType {
        self.casts.last().map_or(self.ty, |c| c.to)
    }

    pub fn new(ty: QualifiedType, kind: ValueCategory) -> Self {
        Self { ty, kind, casts: Vec::new(), result_cast: None, bit_width: None }
    }
}
