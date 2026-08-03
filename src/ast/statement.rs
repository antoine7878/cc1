use crate::arena::{Arena, ArenaId};
use crate::ast::ExpressionNode;
use crate::define_arena;
use crate::parser::Span;

define_arena!(Statement, StatementArena, StatementId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StatementNode {
    span: Span,
    id: StatementId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Statement {
    Labeled(LabeledNode),
    Compound,
    Expression,
    Selection,
    Iteration,
    Jump,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LabeledNode {
    span: Span,
    inner: Labeled,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Labeled {
    IDENTIFIER(StatementNode),
    CASE(ExpressionNode, StatementNode),
    DEFAULT(StatementNode),
}

impl StatementArena {
    pub fn add(&mut self, stmt: Statement, span: Span) -> StatementNode {
        StatementNode {
            id: self.alloc(stmt),
            span,
        }
    }

    pub fn labeled(&mut self, node: LabeledNode, span: Span) -> StatementNode {
        self.add(Statement::Labeled(node), span)
    }
}
