use crate::ast::statement::StatementId;
use crate::ast::visit::{Visitor, walk_jump_statement, walk_labeled_statement};
use crate::ast::{
    ConstFolder, ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, LabeledStatement, LabeledStatementNode, SelectionStatement, SelectionStatementNode,
    StatementNode,
};
use crate::semantic::resolution::expression::{self, operands};
use crate::semantic::{
    AssignmentContext, Diag, Diagnostic, DiagnosticSink, QualifiedType, ResolvedStatement, Resolver, Sema, cast,
};

pub type StmtResult = Result<(), Diagnostic>;

pub fn resolve_labeled_statement(resolver: &mut Resolver, id: StatementId, node: &LabeledStatementNode) {
    check_labeled_statement(resolver, id, node);
    walk_labeled_statement(resolver, node);
}

fn check_labeled_statement(resolver: &mut Resolver, id: StatementId, node: &LabeledStatementNode) {
    let res = match &node.inner {
        LabeledStatement::Identifier(name, _) => {
            resolver.define_label(*name, &node.span);
            Ok(())
        }
        LabeledStatement::Case(expr, _) => check_case(resolver, id, expr),
        LabeledStatement::Default(_) => check_default(resolver, id),
    };
    if let Err(diag) = res {
        resolver.sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_case(resolver: &mut Resolver, id: StatementId, expr: &ExpressionNode) -> StmtResult {
    let value = resolver.eval_constant(expr);
    let Some(control) = resolver.switch_control() else {
        return Err(Diagnostic::OutsideSwitch("case"));
    };
    let Some(value) = value else { return Err(Diagnostic::Poisoned) };
    if value.get_integer_value().is_none() {
        return Err(Diagnostic::NonIntegerConstantExpression);
    }
    let sema = &resolver.sema;
    let ty = sema.types.get(control.id);
    let Some(value) = ConstFolder.convert(ty, value) else {
        return Err(Diagnostic::Poisoned);
    };
    resolver.record_case(value, id)?;
    Ok(())
}

fn check_default(resolver: &mut Resolver, id: StatementId) -> StmtResult {
    resolver.record_default(id)?;
    Ok(())
}

pub fn resolve_selection_statement(resolver: &mut Resolver, id: StatementId, node: &SelectionStatementNode) {
    match &node.stmt {
        SelectionStatement::If(condition, then, otherwise) => {
            resolver.visit_expression(condition);
            check_selection_statement(resolver.sema, node);
            resolver.visit_statement(then);
            if let Some(otherwise) = otherwise {
                resolver.visit_statement(otherwise);
            }
        }
        SelectionStatement::Switch(condition, body) => {
            resolver.visit_expression(condition);
            let control = check_selection_statement(resolver.sema, node);
            resolver.enter_switch(id, control);
            resolver.visit_statement(body);
            resolver.leave_stmt();
        }
    }
}

fn check_selection_statement(sema: &mut Sema, node: &SelectionStatementNode) -> QualifiedType {
    let res = match &node.stmt {
        SelectionStatement::If(e1, _, _) => check_scalar(sema, e1),
        SelectionStatement::Switch(e, _) => check_integral(sema, e),
    };
    match res {
        Err(diag) => {
            sema.add_diag(Diag::err((), diag), &node.span);
            QualifiedType::plain(sema.builtins.int)
        }
        Ok(r) => r,
    }
}

fn expr_to_void(sema: &mut Sema, e: &ExpressionNode) -> Option<()> {
    let mut ops = operands(sema, [e]).ok()?;
    let (sema, [re]) = ops.parts();
    cast::convert(sema, re, sema.builtins.void, false);
    Some(())
}

pub fn resolve_iteration_statement(resolver: &mut Resolver, id: StatementId, node: &IterationStatementNode) {
    let body = loop_controls(resolver, node);
    check_iteration_statement(resolver.sema, node);
    resolver.enter_loop(id);
    resolver.visit_statement(body);
    resolver.leave_stmt();
}

fn loop_controls<'n>(resolver: &mut Resolver, node: &'n IterationStatementNode) -> &'n StatementNode {
    match &node.stmt {
        IterationStatement::While(e, body) | IterationStatement::Do(body, e) => {
            resolver.visit_expression(e);
            body
        }
        IterationStatement::For(b) => {
            let (init, condition, step, body) = &**b;
            for e in [&init.expr, &condition.expr, step].into_iter().flatten() {
                resolver.visit_expression(e);
            }
            body
        }
    }
}

fn check_iteration_statement(sema: &mut Sema, node: &IterationStatementNode) {
    let res = match &node.stmt {
        IterationStatement::While(e, _) => check_scalar(sema, e),
        IterationStatement::Do(_, e) => check_scalar(sema, e),
        IterationStatement::For(b) => check_for(sema, b),
    };
    if let Err(diag) = res {
        sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_scalar(sema: &mut Sema, node: &ExpressionNode) -> Result<QualifiedType, Diagnostic> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::convert_operand(sema, re, &node.span);
    if !re.ty.is_scalar(sema) {
        return Err(Diagnostic::NonScalarStatement(re.ty));
    }
    Ok(re.ty)
}

fn check_integral(sema: &mut Sema, node: &ExpressionNode) -> Result<QualifiedType, Diagnostic> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::convert_operand(sema, re, &node.span);
    if !re.ty.is_integral(sema) {
        return Err(Diagnostic::NonIntegralStatement(re.ty));
    }
    cast::promote(sema, re);
    Ok(re.casted_ty())
}

fn check_for(
    sema: &mut Sema,
    b: &(ExpressionStatementNode, ExpressionStatementNode, Option<ExpressionNode>, StatementNode),
) -> Result<QualifiedType, Diagnostic> {
    let (e1, e2, e3, _) = b;
    e1.expr.as_ref().and_then(|e| expr_to_void(sema, e));
    e3.as_ref().and_then(|e| expr_to_void(sema, e));
    if let Some(e) = e2.expr.as_ref() {
        check_scalar(sema, e)
    } else {
        Ok(QualifiedType::plain(sema.builtins.int))
    }
}

pub fn resolve_jump_statement(resolver: &mut Resolver, id: StatementId, node: &JumpStatementNode) {
    let return_ty = resolver.return_ty();
    walk_jump_statement(resolver, node);
    check_jump_statement(resolver, id, node, return_ty);
}

fn check_jump_statement(
    resolver: &mut Resolver,
    id: StatementId,
    node: &JumpStatementNode,
    return_ty: Option<QualifiedType>,
) {
    let res = match &node.stmt {
        JumpStatement::Goto(name) => {
            resolver.reference_label(*name);
            Ok(())
        }
        JumpStatement::Return(_) => check_return(resolver.sema, node, return_ty.expect("a return inside a function")),
        JumpStatement::Break => check_break(resolver, id),
        JumpStatement::Continue => check_continue(resolver, id),
    };
    if let Err(diag) = res {
        resolver.sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_break(resolver: &mut Resolver, id: StatementId) -> StmtResult {
    let Some(stmt) = resolver.break_target() else {
        return Err(Diagnostic::BreakNotInLoop);
    };
    resolver.sema.statements.set(id, Some(ResolvedStatement::Break(stmt)));
    Ok(())
}

fn check_continue(resolver: &mut Resolver, id: StatementId) -> StmtResult {
    let Some(stmt) = resolver.continue_target() else {
        return Err(Diagnostic::ContinueNotInLoop);
    };
    resolver.sema.statements.set(id, Some(ResolvedStatement::Continue(stmt)));
    Ok(())
}

fn check_return(sema: &mut Sema, node: &JumpStatementNode, return_ty: QualifiedType) -> StmtResult {
    match &node.stmt {
        JumpStatement::Return(Some(e)) => expression::init(sema, return_ty, e, AssignmentContext::Return).map(|_| ()),
        JumpStatement::Return(None) if return_ty.id != sema.builtins.void => Err(Diagnostic::ReturnWithoutValue),
        _ => Ok(()),
    }
}
