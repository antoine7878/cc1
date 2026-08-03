use crate::arena::{Arena, ArenaId};
use crate::ast::{DeclarationNode, ExpressionNode, Name};
use crate::define_arena;
use crate::parser::Span;

define_arena!(Statement, StatementArena, StatementId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StatementNode {
    pub span: Span,
    pub id: StatementId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Statement {
    Labeled(LabeledStatementNode),
    Compound(CompoundStatementNode),
    Expression(ExpressionStatementNode),
    Selection(SelectionStatementNode),
    Iteration(IterationStatementNode),
    Jump(JumpStatementNode),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LabeledStatementNode {
    pub span: Span,
    pub inner: Labeled,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Labeled {
    Identifier(Name, StatementNode),
    Case(ExpressionNode, StatementNode),
    Default(StatementNode),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CompoundStatementNode {
    pub span: Span,
    pub declarations: Vec<DeclarationNode>,
    pub statements: Vec<StatementNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ExpressionStatementNode {
    pub span: Span,
    pub expr: Option<ExpressionNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SelectionStatementNode {
    span: Span,
    stmt: SelectionStatement,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SelectionStatement {
    If(ExpressionNode, StatementNode, Option<StatementNode>),
    Switch(ExpressionNode, StatementNode),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IterationStatementNode {
    span: Span,
    stmt: IterationStatement,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum IterationStatement {
    While(ExpressionNode, StatementNode),
    Do(StatementNode, ExpressionNode),
    For(
        ExpressionStatementNode,
        ExpressionStatementNode,
        Option<ExpressionNode>,
        StatementNode,
    ),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct JumpStatementNode {
    span: Span,
    stmt: JumpStatement,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum JumpStatement {
    Goto,
    Continue,
    Break,
    Return(Option<ExpressionNode>),
}

impl StatementArena {
    pub fn add(&mut self, stmt: Statement, span: Span) -> StatementNode {
        StatementNode {
            id: self.alloc(stmt),
            span,
        }
    }

    pub fn labeled(&mut self, node: LabeledStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Labeled(node), span)
    }

    pub fn compund(&mut self, stmt: CompoundStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Compound(stmt), span)
    }

    pub fn expression(&mut self, stmt: ExpressionStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Expression(stmt), span)
    }

    pub fn selection(&mut self, stmt: SelectionStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Selection(stmt), span)
    }

    pub fn iteration(&mut self, stmt: IterationStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Iteration(stmt), span)
    }

    pub fn jump(&mut self, stmt: JumpStatementNode, span: Span) -> StatementNode {
        self.add(Statement::Jump(stmt), span)
    }
}

impl LabeledStatementNode {
    pub fn new(inner: Labeled, span: Span) -> LabeledStatementNode {
        LabeledStatementNode { inner, span }
    }

    pub fn identifier(name: Name, stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(Labeled::Identifier(name, stmt), span)
    }

    pub fn case(node: ExpressionNode, stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(Labeled::Case(node, stmt), span)
    }

    pub fn default(stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(Labeled::Default(stmt), span)
    }
}

impl CompoundStatementNode {
    pub fn new(
        declarations: Vec<DeclarationNode>,
        statements: Vec<StatementNode>,
        span: Span,
    ) -> CompoundStatementNode {
        CompoundStatementNode {
            declarations,
            statements,
            span,
        }
    }
}

impl ExpressionStatementNode {
    pub fn new(expr: Option<ExpressionNode>, span: Span) -> ExpressionStatementNode {
        ExpressionStatementNode { expr, span }
    }
}

impl SelectionStatementNode {
    pub fn new_if(
        expr: ExpressionNode,
        sif: StatementNode,
        selse: Option<StatementNode>,
        span: Span,
    ) -> SelectionStatementNode {
        SelectionStatementNode {
            span,
            stmt: SelectionStatement::If(expr, sif, selse),
        }
    }

    pub fn switch(expr: ExpressionNode, stmt: StatementNode, span: Span) -> SelectionStatementNode {
        SelectionStatementNode {
            span,
            stmt: SelectionStatement::Switch(expr, stmt),
        }
    }
}

impl IterationStatementNode {
    pub fn new_while(expr: ExpressionNode, stmt: StatementNode, span: Span) -> IterationStatementNode {
        IterationStatementNode {
            span,
            stmt: IterationStatement::While(expr, stmt),
        }
    }

    pub fn new_do(stmt: StatementNode, expr: ExpressionNode, span: Span) -> IterationStatementNode {
        IterationStatementNode {
            span,
            stmt: IterationStatement::Do(stmt, expr),
        }
    }

    pub fn new_for(
        e1: ExpressionStatementNode,
        e2: ExpressionStatementNode,
        expr: Option<ExpressionNode>,
        stmt: StatementNode,
        span: Span,
    ) -> IterationStatementNode {
        IterationStatementNode {
            span,
            stmt: IterationStatement::For(e1, e2, expr, stmt),
        }
    }
}

impl JumpStatementNode {
    pub fn new(stmt: JumpStatement, span: Span) -> JumpStatementNode {
        JumpStatementNode { span, stmt }
    }
    pub fn new_return(expr: Option<ExpressionNode>, span: Span) -> JumpStatementNode {
        JumpStatementNode {
            span,
            stmt: JumpStatement::Return(expr),
        }
    }
}
