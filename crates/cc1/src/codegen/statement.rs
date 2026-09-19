use std::io::Write;

use crate::ast::statement::StatementId;
use crate::ast::{
    ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, LabeledStatement, LabeledStatementNode, SelectionStatement, SelectionStatementNode,
    StatementNode, Visitor,
};
use crate::codegen::{Generator, LlvmName, LlvmSymbol};
use crate::semantic::{Diagnostic, ResolvedStatement, sema};

impl<W: Write> Generator<W> {
    pub fn emit_selection_statement(
        &mut self,
        id: StatementId,
        node: &SelectionStatementNode,
    ) -> Result<(), Diagnostic> {
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
    ) -> Result<(), Diagnostic> {
        let cond = self.emit_condition(condition)?;
        let then_l = self.builder.fresh_label();
        let else_l = self.builder.fresh_label();
        let join_l = if otherwise_body.is_some() { self.builder.fresh_label() } else { else_l };

        self.builder.br_cond(cond, then_l, else_l);
        self.builder.label(then_l);
        self.visit_statement(then_body);
        self.builder.br(join_l);

        if let Some(otherwise) = otherwise_body {
            self.builder.label(else_l);
            self.visit_statement(otherwise);
            self.builder.br(join_l);
        }
        self.builder.label(join_l);
        Ok(())
    }

    fn switch_statement(
        &mut self,
        id: StatementId,
        condition: &ExpressionNode,
        body: &StatementNode,
    ) -> Result<(), Diagnostic> {
        let ResolvedStatement::Switch { control, cases, default } = &sema().statements[id] else {
            return Err(Diagnostic::Invariant("switch"));
        };
        let end_l = LlvmName::BreakLabel(id);
        let condition = self.emit_expression(condition)?;
        let llvm_cases = cases
            .iter()
            .map(|(v, id)| (LlvmSymbol::new(control.llvm(), v.llvm()), LlvmName::CaseLabel(*id)))
            .collect::<Vec<_>>();
        let llvm_default = default.map(LlvmName::CaseLabel);
        self.builder.switch(condition, &llvm_cases, llvm_default.unwrap_or(end_l));
        self.visit_statement(body);
        self.builder.br(end_l);
        self.builder.label(end_l);
        Ok(())
    }
    pub fn emit_iteration_statement(
        &mut self,
        id: StatementId,
        node: &IterationStatementNode,
    ) -> Result<(), Diagnostic> {
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
    ) -> Result<(), Diagnostic> {
        self.emit_loop(id, false, Some(condition), None, body)
    }

    fn do_statement(
        &mut self,
        id: StatementId,
        body: &StatementNode,
        condition: &ExpressionNode,
    ) -> Result<(), Diagnostic> {
        self.emit_loop(id, true, Some(condition), None, body)
    }

    fn for_statement(
        &mut self,
        id: StatementId,
        b: &(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode),
    ) -> Result<(), Diagnostic> {
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
    ) -> Result<(), Diagnostic> {
        let cond_l = self.builder.fresh_label();
        let body_l = self.builder.fresh_label();
        let action_l = LlvmName::ContinueLabel(id);
        let end_l = LlvmName::BreakLabel(id);

        self.builder.br(if enter_at_body { body_l } else { cond_l });
        self.builder.label(cond_l);
        match condition {
            Some(cond) => {
                let cond = self.emit_condition(cond)?;
                self.builder.br_cond(cond, body_l, end_l);
            }
            None => self.builder.br(body_l),
        }
        self.builder.label(body_l);
        self.visit_statement(body);
        self.builder.br(action_l);
        self.builder.label(action_l);
        if let Some(action) = action {
            self.emit_expression(action)?;
        }
        self.builder.br(cond_l);
        self.builder.label(end_l);
        Ok(())
    }

    pub fn emit_jump_statement(&mut self, id: StatementId, node: &JumpStatementNode) -> Result<(), Diagnostic> {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => self.return_(e)?,
            JumpStatement::Return(None) => self.builder.ret_void(),
            JumpStatement::Break => {
                let &ResolvedStatement::Break(target) = &sema().statements[id] else {
                    return Err(Diagnostic::Invariant("break target"));
                };
                self.builder.br(LlvmName::BreakLabel(target));
            }
            JumpStatement::Continue => {
                let &ResolvedStatement::Continue(target) = &sema().statements[id] else {
                    return Err(Diagnostic::Invariant("continue target"));
                };
                self.builder.br(LlvmName::ContinueLabel(target));
            }
            JumpStatement::Goto(a) => self.builder.br(LlvmName::NamedLabel(a.id)),
        }
        Ok(())
    }

    fn return_(&mut self, node: &ExpressionNode) -> Result<(), Diagnostic> {
        let qty = sema().expressions[node.id].casted_ty();
        match self.locals.sret {
            Some(dst) => {
                self.emit_copy_aggregate(dst, node, qty)?;
                self.builder.ret_void();
            }
            None => {
                let v = self.emit_expression(node)?;
                self.builder.ret(v);
            }
        }
        Ok(())
    }

    pub fn emit_labeled_statement(&mut self, id: StatementId, node: &LabeledStatementNode) -> Result<(), Diagnostic> {
        let (l, stmt) = match &node.inner {
            LabeledStatement::Identifier(label, stmt) => (LlvmName::NamedLabel(label.id), stmt),
            LabeledStatement::Case(_, stmt) | LabeledStatement::Default(stmt) => (LlvmName::CaseLabel(id), stmt),
        };
        self.builder.br(l);
        self.builder.label(l);
        self.visit_statement(stmt);
        Ok(())
    }
}
