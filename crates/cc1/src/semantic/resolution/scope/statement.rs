use crate::ast::ConstValue;
use crate::ast::statement::StatementId;
use crate::semantic::{Diagnosis, QualifiedType, ResolvedStatement};

#[derive(Debug)]
pub enum StatementScope {
    Loop(StatementId),
    Switch {
        stmt: StatementId,
        control: QualifiedType,
        cases: Vec<(ConstValue, StatementId)>,
        default: Option<StatementId>,
    },
}

impl StatementScope {
    fn stmt(&self) -> StatementId {
        match self {
            StatementScope::Loop(stmt) => *stmt,
            StatementScope::Switch { stmt, .. } => *stmt,
        }
    }
}

#[derive(Default, Debug)]
pub struct StatementScopes(Vec<StatementScope>);

impl StatementScopes {
    pub fn push_loop(&mut self, stmt: StatementId) {
        self.0.push(StatementScope::Loop(stmt))
    }

    pub fn push_switch(&mut self, stmt: StatementId, control: QualifiedType) {
        self.0.push(StatementScope::Switch { stmt, control, cases: Vec::new(), default: None })
    }

    pub fn pop(&mut self) -> Option<(StatementId, ResolvedStatement)> {
        self.0.pop().map(Into::into)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn breakable(&self) -> Option<StatementId> {
        self.0.last().map(StatementScope::stmt)
    }

    pub fn nearest_loop(&self) -> Option<StatementId> {
        self.0.iter().rev().find_map(|scope| match scope {
            StatementScope::Loop(stmt) => Some(*stmt),
            StatementScope::Switch { .. } => None,
        })
    }

    pub fn switch_control(&self) -> Option<QualifiedType> {
        self.0.iter().rev().find_map(|scope| match scope {
            StatementScope::Switch { control, .. } => Some(*control),
            StatementScope::Loop(_) => None,
        })
    }

    pub fn record_case(&mut self, value: ConstValue, id: StatementId) -> Result<StatementId, Diagnosis> {
        let Some(StatementScope::Switch { stmt, cases, .. }) = self.nearest_switch() else {
            return Err(Diagnosis::OutsideSwitch("case"));
        };
        if cases.iter().any(|(v, _)| value == *v) {
            return Err(Diagnosis::DuplicateCase(value));
        }
        cases.push((value, id));
        Ok(*stmt)
    }

    pub fn record_default(&mut self, id: StatementId) -> Result<StatementId, Diagnosis> {
        let Some(StatementScope::Switch { stmt, default, .. }) = self.nearest_switch() else {
            return Err(Diagnosis::OutsideSwitch("default"));
        };
        if default.is_some() {
            return Err(Diagnosis::DuplicateDefault);
        }
        *default = Some(id);
        Ok(*stmt)
    }

    fn nearest_switch(&mut self) -> Option<&mut StatementScope> {
        self.0.iter_mut().rev().find(|scope| matches!(scope, StatementScope::Switch { .. }))
    }
}

impl From<StatementScope> for (StatementId, ResolvedStatement) {
    fn from(value: StatementScope) -> Self {
        match value {
            StatementScope::Switch { stmt, control, cases, default } => {
                (stmt, ResolvedStatement::Switch { control, cases, default })
            }
            StatementScope::Loop(stmt) => (stmt, ResolvedStatement::Loop(stmt)),
        }
    }
}
