use std::io::Write;

use crate::ast::statement::StatementId;
use crate::ast::{
    ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, LabeledStatement, LabeledStatementNode, SelectionStatement, SelectionStatementNode,
    StatementNode, Visitor,
};
use crate::codegen::{Generator, LlvmName, LlvmSymbol};
use crate::semantic::{Diagnosis, ResolvedStatement, sema};

impl<W: Write> Generator<W> {
    pub fn selection_statement(&mut self, id: StatementId, node: &SelectionStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            SelectionStatement::If(cond, then, otherwise) => self.if_statement(cond, then, otherwise),
            SelectionStatement::Switch(condition, body) => self.switch_statement(id, condition, body),
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

    fn switch_statement(
        &mut self,
        id: StatementId,
        condition: &ExpressionNode,
        body: &StatementNode,
    ) -> Result<(), Diagnosis> {
        let ResolvedStatement::Switch { control, cases, default } = &sema().stmts[id] else {
            return Err(Diagnosis::Invariant("switch"));
        };
        let end_l = LlvmName::BreakLabel(id);
        let condition = self.emit_expression(condition)?;
        let llvm_cases = cases
            .iter()
            .map(|(v, id)| (LlvmSymbol::new(control.llvm(), v.llvm()), LlvmName::CaseLabel(*id)))
            .collect::<Vec<_>>();
        let llvm_default = default.map(LlvmName::CaseLabel);
        self.b.switch(condition, &llvm_cases, llvm_default.unwrap_or(end_l));
        self.visit_statement(body);
        self.b.br(end_l);
        self.b.emit_label(end_l);
        Ok(())
    }
    pub fn iteration_statement(&mut self, id: StatementId, node: &IterationStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            IterationStatement::While(cond, stmt) => self.while_statement(id, cond, stmt),
            IterationStatement::Do(stmt, cond) => self.do_statement(id, stmt, cond),
            IterationStatement::For(b) => self.for_statement(id, b),
        }
    }

    fn while_statement(
        &mut self,
        id: StatementId,
        condition: &ExpressionNode,
        body: &StatementNode,
    ) -> Result<(), Diagnosis> {
        self.emit_loop(id, false, Some(condition), None, body)
    }

    fn do_statement(
        &mut self,
        id: StatementId,
        body: &StatementNode,
        condition: &ExpressionNode,
    ) -> Result<(), Diagnosis> {
        self.emit_loop(id, true, Some(condition), None, body)
    }

    fn for_statement(
        &mut self,
        id: StatementId,
        b: &(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode),
    ) -> Result<(), Diagnosis> {
        let (init, cond, action, body) = b;
        if let Some(init) = &init.expr {
            self.emit_expression(init)?;
        }
        self.emit_loop(id, false, cond.expr.as_ref(), action.as_ref(), body)
    }

    fn emit_loop(
        &mut self,
        id: StatementId,
        enter_at_body: bool,
        condition: Option<&ExpressionNode>,
        action: Option<&ExpressionNode>,
        body: &StatementNode,
    ) -> Result<(), Diagnosis> {
        let cond_l = self.b.fresh_label();
        let body_l = self.b.fresh_label();
        let action_l = LlvmName::ContinueLabel(id);
        let end_l = LlvmName::BreakLabel(id);

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
        self.visit_statement(body);
        self.b.br(action_l);
        self.b.emit_label(action_l);
        if let Some(action) = action {
            self.emit_expression(action)?;
        }
        self.b.br(cond_l);
        self.b.emit_label(end_l);
        Ok(())
    }

    pub fn jump_statement(&mut self, id: StatementId, node: &JumpStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => self.emit_expression(e).map(|v| self.b.ret(v))?,
            JumpStatement::Return(None) => self.b.ret_void(),
            JumpStatement::Break => {
                let &ResolvedStatement::Break(target) = &sema().stmts[id] else {
                    return Err(Diagnosis::Invariant("break target"));
                };
                self.b.br(LlvmName::BreakLabel(target));
            }
            JumpStatement::Continue => {
                let &ResolvedStatement::Continue(target) = &sema().stmts[id] else {
                    return Err(Diagnosis::Invariant("continue target"));
                };
                self.b.br(LlvmName::ContinueLabel(target));
            }
            JumpStatement::Goto(a) => self.b.br(LlvmName::NamedLabel(a.id)),
        }
        Ok(())
    }

    pub fn labeled_statement(&mut self, id: StatementId, node: &LabeledStatementNode) -> Result<(), Diagnosis> {
        let (l, stmt) = match &node.inner {
            LabeledStatement::Identifier(label, stmt) => (LlvmName::NamedLabel(label.id), stmt),
            LabeledStatement::Case(_, stmt) | LabeledStatement::Default(stmt) => (LlvmName::CaseLabel(id), stmt),
        };
        self.b.br(l);
        self.b.emit_label(l);
        self.visit_statement(stmt);
        Ok(())
    }
}
