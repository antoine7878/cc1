use crate::{
    ast::{StringId, Value, statement::StatementId},
    semantic::QualifiedType,
};

#[derive(Debug)]
pub enum ResolvedStatement {
    Loop(StatementId),
    Switch {
        control: QualifiedType,
        cases: Vec<(Value, StatementId)>,
        default: Option<StatementId>,
    },
    Case(Value, StatementId),
    Default(StatementId),
    Break(StatementId),
    Continue(StatementId),
    Goto(StringId),
    Label(StringId),
}
