use crate::{
    ast::{ConstValue, StringId, statement::StatementId},
    semantic::QualifiedType,
};

#[derive(Debug)]
pub enum ResolvedStatement {
    Loop(StatementId),
    Switch {
        control: QualifiedType,
        cases: Vec<(ConstValue, StatementId)>,
        default: Option<StatementId>,
    },
    Case(ConstValue, StatementId),
    Default(StatementId),
    Break(StatementId),
    Continue(StatementId),
    Goto(StringId),
    Label(StringId),
}
