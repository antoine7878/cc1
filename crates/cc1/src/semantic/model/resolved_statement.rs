use crate::ast::ConstValue;
use crate::ast::statement::StatementId;
use crate::semantic::QualifiedType;

#[derive(Debug)]
pub enum ResolvedStatement {
    Switch { control: QualifiedType, cases: Vec<(ConstValue, StatementId)>, default: Option<StatementId> },
    Break(StatementId),
    Continue(StatementId),
}
