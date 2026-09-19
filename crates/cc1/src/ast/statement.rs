use libft::Span;

use crate::ast::{DeclarationNode, ExpressionNode, Name};
use crate::{ast_node, define_arena};

define_arena!(Statement, StatementArena, StatementId);

ast_node! {
    pub struct StatementNode {
        pub id: StatementId,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Labeled(LabeledStatementNode),
    Selection(SelectionStatementNode),
    Iteration(IterationStatementNode),
    Jump(JumpStatementNode),
    Expression(ExpressionStatementNode),
    Compound(CompoundStatementNode),
}

ast_node! {
    pub struct LabeledStatementNode {
        pub inner: LabeledStatement,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LabeledStatement {
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

#[derive(Clone, Debug, PartialEq)]
pub enum SelectionStatement {
    If(ExpressionNode, StatementNode, Option<StatementNode>),
    Switch(ExpressionNode, StatementNode),
}

ast_node! {
    pub struct IterationStatementNode {
        pub stmt: IterationStatement,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IterationStatement {
    While(ExpressionNode, StatementNode),
    Do(StatementNode, ExpressionNode),
    For(Box<(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode)>),
}

ast_node! {
    pub struct JumpStatementNode {
        pub stmt: JumpStatement,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum JumpStatement {
    Goto(Name),
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

    pub fn compound(&mut self, stmt: CompoundStatementNode, span: Span) -> StatementNode {
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
    pub fn identifier_label(name: Name, stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(LabeledStatement::Identifier(name, stmt), span)
    }

    pub fn case_label(node: ExpressionNode, stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(LabeledStatement::Case(node, stmt), span)
    }

    pub fn default_label(stmt: StatementNode, span: Span) -> LabeledStatementNode {
        Self::new(LabeledStatement::Default(stmt), span)
    }
}
impl SelectionStatementNode {
    pub fn if_stmt(
        e_condition: ExpressionNode,
        s_if: StatementNode,
        s_else: Option<StatementNode>,
        span: Span,
    ) -> SelectionStatementNode {
        SelectionStatementNode::new(SelectionStatement::If(e_condition, s_if, s_else), span)
    }

    pub fn switch_stmt(expr: ExpressionNode, stmt: StatementNode, span: Span) -> SelectionStatementNode {
        SelectionStatementNode::new(SelectionStatement::Switch(expr, stmt), span)
    }
}

impl IterationStatementNode {
    pub fn while_stmt(expr: ExpressionNode, stmt: StatementNode, span: Span) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::While(expr, stmt), span)
    }

    pub fn do_stmt(stmt: StatementNode, expr: ExpressionNode, span: Span) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::Do(stmt, expr), span)
    }

    pub fn for_stmt(
        e1: ExpressionStatementNode,
        e2: ExpressionStatementNode,
        expr: Option<ExpressionNode>,
        stmt: StatementNode,
        span: Span,
    ) -> IterationStatementNode {
        IterationStatementNode::new(IterationStatement::For(Box::new((e1, e2, expr, stmt))), span)
    }
}

impl JumpStatementNode {
    pub fn goto_stmt(name: Name, span: Span) -> JumpStatementNode {
        JumpStatementNode::new(JumpStatement::Goto(name), span)
    }
    pub fn return_stmt(expr: Option<ExpressionNode>, span: Span) -> JumpStatementNode {
        JumpStatementNode::new(JumpStatement::Return(expr), span)
    }
}
