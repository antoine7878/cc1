use crate::{
    ast::{Value, statement::StatementId},
    semantic::{QualifiedType, SymbolId},
};

#[derive(Debug)]
pub enum ResolvedStatement {
    Loop {
        break_target: StatementId,
        continue_target: StatementId,
        reachable: bool,
    },
    Switch {
        control_ty: QualifiedType,
        cases: Vec<(Value, StatementId)>,
        default: Option<StatementId>,
    },
    Case(Value, StatementId),
    Default(SymbolId),
    Break(SymbolId),
    Continue(SymbolId),
    Goto(SymbolId),
    Label(SymbolId),
}
