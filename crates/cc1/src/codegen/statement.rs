use std::io::Write;

use crate::ast::{
    ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, LabeledStatement, LabeledStatementNode, SelectionStatement, SelectionStatementNode,
    StatementNode, Visitor,
};
use crate::codegen::{Generator, LlvmName};
use crate::semantic::{Diagnosis, sema};

impl<W: Write> Generator<W> {
    pub fn selection_statement(
        &mut self,
        node: &StatementNode,
        selection_node: &SelectionStatementNode,
    ) -> Result<(), Diagnosis> {
        let rs = &sema().stmts[node.id];
        match &selection_node.stmt {
            SelectionStatement::If(cond, then, otherwise) => self.if_statement(cond, then, otherwise),
            SelectionStatement::Switch(cond, stmt) => self.switch_statement(rs),
        }
    }

    fn if_statement(
        &mut self,
        condition: &ExpressionNode,
        then_body: &StatementNode,
        otherwise_body: &Option<StatementNode>,
    ) -> Result<(), Diagnosis> {
        let cond = self.emit_condition(condition)?;
        let then_l = self.b.fresh_label();
        let else_l = self.b.fresh_label();
        let join_l = if otherwise_body.is_some() { self.b.fresh_label() } else { else_l };

        self.b.brc(cond, then_l, else_l);
        self.b.emit_label(then_l);
        self.visit_statement(then_body);
        self.b.br(join_l);

        if let Some(otherwise) = otherwise_body {
            self.b.emit_label(else_l);
            self.visit_statement(otherwise);
            self.b.br(join_l);
        }
        self.b.emit_label(join_l);
        Ok(())
    }

    fn switch_statement(&mut self, condition: &ExpressionNode, body: &StatementNode) -> Result<(), Diagnosis> {
        // let rs = sema().stmts[]
        Ok(())
    }

    pub fn iteration_statement(&mut self, node: &IterationStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            IterationStatement::While(cond, stmt) => self.while_statement(cond, stmt),
            IterationStatement::Do(stmt, cond) => self.do_statement(stmt, cond),
            IterationStatement::For(b) => self.for_statement(b),
        }
    }

    fn in_loop<R>(&mut self, continue_l: LlvmName, break_l: LlvmName, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved = (self.continue_label, self.break_label);
        self.continue_label = continue_l;
        self.break_label = break_l;
        let r = f(self);
        (self.continue_label, self.break_label) = saved;
        r
    }

    fn while_statement(&mut self, condition: &ExpressionNode, body: &StatementNode) -> Result<(), Diagnosis> {
        self.emit_loop(false, Some(condition), None, body)
    }

    fn do_statement(&mut self, body: &StatementNode, condition: &ExpressionNode) -> Result<(), Diagnosis> {
        self.emit_loop(true, Some(condition), None, body)
    }

    fn for_statement(
        &mut self,
        b: &(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode),
    ) -> Result<(), Diagnosis> {
        let (init, cond, action, body) = b;
        if let Some(init) = &init.expr {
            self.emit_expression(init)?;
        }
        self.emit_loop(false, cond.expr.as_ref(), action.as_ref(), body)
    }

    fn emit_loop(
        &mut self,
        enter_at_body: bool,
        condition: Option<&ExpressionNode>,
        action: Option<&ExpressionNode>,
        body: &StatementNode,
    ) -> Result<(), Diagnosis> {
        let cond_l = self.b.fresh_label();
        let body_l = self.b.fresh_label();
        let action_l = self.b.fresh_label();
        let end_l = self.b.fresh_label();

        self.b.br(if enter_at_body { body_l } else { cond_l });
        self.b.emit_label(cond_l);
        match condition {
            Some(cond) => {
                let cond = self.emit_condition(cond)?;
                self.b.brc(cond, body_l, end_l);
            }
            None => self.b.br(body_l),
        }
        self.b.emit_label(body_l);
        self.in_loop(action_l, end_l, |g| g.visit_statement(body));
        self.b.br(action_l);
        self.b.emit_label(action_l);
        if let Some(action) = action {
            self.emit_expression(action)?;
        }
        self.b.br(cond_l);
        self.b.emit_label(end_l);
        Ok(())
    }

    pub fn jump_statement(&mut self, node: &JumpStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => self.emit_expression(e).map(|v| self.b.ret(v))?,
            JumpStatement::Return(None) => self.b.ret_void(),
            JumpStatement::Break => self.b.br(self.break_label),
            JumpStatement::Continue => self.b.br(self.continue_label),
            JumpStatement::Goto(a) => self.b.br(LlvmName::NamedLabel(a.id)),
        }
        Ok(())
    }

    pub fn labeled_statement(&mut self, node: &LabeledStatementNode) -> Result<(), Diagnosis> {
        let stmt = match &node.inner {
            LabeledStatement::Identifier(label, stmt) => {
                let l = LlvmName::NamedLabel(label.id);
                self.b.br(l);
                self.b.emit_label(l);
                stmt
            }
            LabeledStatement::Default(stmt) => stmt,
            LabeledStatement::Case(_, stmt) => stmt,
        };
        self.visit_statement(stmt);
        Ok(())
    }
}
