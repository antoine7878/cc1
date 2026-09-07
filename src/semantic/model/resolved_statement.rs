use crate::{
    ast::{StringId, Value, statement::StatementId},
    semantic::QualifiedType,
};

#[derive(Debug)]
pub enum StatementScope {
    Loop(StatementId),
    Switch {
        stmt: StatementId,
        control: QualifiedType,
        cases: Vec<(Value, StatementId)>,
        default: Option<StatementId>,
    },
}

#[derive(Default, Debug)]
pub struct StatementScopes(Vec<StatementScope>);

impl StatementScopes {
    pub fn push_loop(&mut self, stmt: StatementId) {
        self.0.push(StatementScope::Loop(stmt))
    }

    pub fn push_switch(
        &mut self,
        stmt: StatementId,
        control: QualifiedType,
        cases: Vec<(Value, StatementId)>,
        default: Option<StatementId>,
    ) {
        self.0.push(StatementScope::Switch {
            stmt,
            control,
            cases,
            default,
        })
    }

    pub fn pop(&mut self) -> Option<StatementScope> {
        self.0.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn last(&self) -> Option<&StatementScope> {
        self.0.last()
    }

    pub fn nearest_loop(&self) -> Option<StatementId> {
        self.0.iter().rev().find_map(|scope| match scope {
            StatementScope::Loop(stmt) => Some(*stmt),
            StatementScope::Switch { .. } => None,
        })
    }

    pub fn nearest_switch(&mut self) -> Option<&mut StatementScope> {
        self.0
            .iter_mut()
            .rev()
            .find(|scope| matches!(scope, StatementScope::Switch { .. }))
    }
}

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

impl From<StatementScope> for (StatementId, ResolvedStatement) {
    fn from(value: StatementScope) -> Self {
        match value {
            StatementScope::Switch {
                stmt,
                control,
                cases,
                default,
            } => (
                stmt,
                ResolvedStatement::Switch {
                    control,
                    cases,
                    default,
                },
            ),
            StatementScope::Loop(stmt) => (stmt, ResolvedStatement::Loop(stmt)),
        }
    }
}
