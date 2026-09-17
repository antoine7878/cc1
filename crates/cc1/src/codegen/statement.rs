use std::io::Write;

use crate::ast::{
    ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, SelectionStatement,
    SelectionStatementNode, StatementNode, Visitor,
};
use crate::codegen::Generator;
use crate::semantic::Diagnosis;

impl<W: Write> Generator<W> {
    pub fn selection_statement(&mut self, node: &SelectionStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            SelectionStatement::If(cond, then, otherwise) => self.if_statement(cond, then, otherwise),
            SelectionStatement::Switch(cond, stmt) => self.switch_statement(cond, stmt),
        }
    }

    fn if_statement(
        &mut self,
        cond: &ExpressionNode,
        then: &StatementNode,
        otherwise: &Option<StatementNode>,
    ) -> Result<(), Diagnosis> {
        let cond = self.emit_condition(cond)?;
        let then_l = self.b.fresh_label();
        let else_l = self.b.fresh_label();
        let join_l = if otherwise.is_some() { self.b.fresh_label() } else { else_l };

        self.b.brc(cond, then_l, else_l);
        self.b.emit_label(then_l);
        self.visit_statement(then);
        self.b.br(join_l);

        if let Some(otherwise) = otherwise {
            self.b.emit_label(else_l);
            self.visit_statement(otherwise);
            self.b.br(join_l);
        }
        self.b.emit_label(join_l);
        Ok(())
    }

    fn switch_statement(&mut self, cond: &ExpressionNode, stmt: &StatementNode) -> Result<(), Diagnosis> {
        todo!()
    }

    pub fn iteration_statement(&mut self, node: &IterationStatementNode) -> Result<(), Diagnosis> {
        match &node.stmt {
            IterationStatement::While(cond, stmt) => self.while_statement(cond, stmt),
            IterationStatement::Do(stmt, cond) => self.do_statement(stmt, cond),
            IterationStatement::For(b) => self.for_statement(b),
        }
    }

    fn while_statement(&mut self, cond: &ExpressionNode, stmt: &StatementNode) -> Result<(), Diagnosis> {
        //   br label %3
        //
        // 3:                                                ; preds = %6, %0
        //   %4 = load i32, ptr %2, align 4
        //   %5 = icmp slt i32 %4, 10
        //   br i1 %5, label %6, label %9
        //
        // 6:                                                ; preds = %3
        //   %7 = load i32, ptr %2, align 4
        //   %8 = add nsw i32 %7, 1
        //   store i32 %8, ptr %2, align 4
        //   br label %3
        //
        // 9:
        let cond_l = self.b.fresh_label();
        let start_l = self.b.fresh_label();
        let end_l = self.b.fresh_label();

        self.b.br(cond_l);
        self.b.emit_label(cond_l);
        let cond = self.emit_condition(cond)?;
        self.b.brc(cond, start_l, end_l);
        self.b.emit_label(start_l);
        self.visit_statement(stmt);
        self.b.br(cond_l);
        self.b.emit_label(end_l);
        Ok(())
    }

    fn do_statement(&mut self, stmt: &StatementNode, cond: &ExpressionNode) -> Result<(), Diagnosis> {
        todo!()
    }

    fn for_statement(
        &mut self,
        b: &(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode),
    ) -> Result<(), Diagnosis> {
        todo!()
    }
}
