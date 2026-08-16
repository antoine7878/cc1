use crate::arena::{Arena, ArenaId};
use crate::ast::{DeclarationNode, ExpressionNode, Name, Node};
use crate::parser::Span;
use crate::{ast_node, define_arena};

define_arena!(Statement, StatementArena, StatementId);

ast_node! {
    pub struct StatementNode {
        pub id: StatementId,
    }
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

ast_node! {
    pub struct LabeledStatementNode {
        pub inner: Labeled,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Labeled {
    Identifier(Name, StatementNode),
    Case(ExpressionNode, StatementNode),
    Default(StatementNode),
}

ast_node! {
    pub struct CompoundStatementNode {
        pub declarations: Vec<DeclarationNode>,
        pub statements: Vec<StatementNode>,
    }
}

ast_node! {
    pub struct ExpressionStatementNode {
        pub expr: Option<ExpressionNode>,
    }
}

ast_node! {
    pub struct SelectionStatementNode {
        pub stmt: SelectionStatement,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SelectionStatement {
    If(ExpressionNode, StatementNode, Option<StatementNode>),
    Switch(ExpressionNode, StatementNode),
}

ast_node! {
pub struct IterationStatementNode {
    pub stmt: IterationStatement,
}
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

ast_node! {
    pub struct JumpStatementNode {
        pub stmt: JumpStatement,
    }
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
        StatementNode::new(self.alloc(stmt), span)
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

impl SelectionStatementNode {
    pub fn new_if(
        expr: ExpressionNode,
        sif: StatementNode,
        selse: Option<StatementNode>,
        span: Span,
    ) -> SelectionStatementNode {
        SelectionStatementNode::new(SelectionStatement::If(expr, sif, selse), span)
    }

    pub fn switch(expr: ExpressionNode, stmt: StatementNode, span: Span) -> SelectionStatementNode {
        SelectionStatementNode::new(SelectionStatement::Switch(expr, stmt), span)
    }
}

impl IterationStatementNode {
    pub fn new_while(expr: ExpressionNode, stmt: StatementNode, span: Span) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::While(expr, stmt), span)
    }

    pub fn new_do(stmt: StatementNode, expr: ExpressionNode, span: Span) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::Do(stmt, expr), span)
    }

    pub fn new_for(
        e1: ExpressionStatementNode,
        e2: ExpressionStatementNode,
        expr: Option<ExpressionNode>,
        stmt: StatementNode,
        span: Span,
    ) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::For(e1, e2, expr, stmt), span)
    }
}

impl JumpStatementNode {
    pub fn new_return(expr: Option<ExpressionNode>, span: Span) -> JumpStatementNode {
        JumpStatementNode::new(JumpStatement::Return(expr), span)
    }
}
