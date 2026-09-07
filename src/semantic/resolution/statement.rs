use crate::ast::statement::StatementId;
use crate::ast::{
    ExpressionNode, ExpressionStatementNode, Fold, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, Labeled, LabeledStatementNode, SelectionStatement, SelectionStatementNode, StatementNode,
};
use crate::context::Context;
use crate::semantic::resolution::expression::{self, operands};
use crate::semantic::resolved_statement::StatementScope;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, QualifiedType, ResolvedStatement, ScopeKind, Sema,
    SymbolResolver, cast,
};

pub type R = Result<(), Diagnosis>;

pub fn check_labeled_statement(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    id: StatementId,
    node: &LabeledStatementNode,
) {
    let res = match &node.inner {
        Labeled::Identifier(name, _) => {
            resolver.define_label(*name, &node.span);
            resolver.sema.stmts.set(id, Some(ResolvedStatement::Label(name.id)));
            Ok(())
        }
        Labeled::Case(expr, _) => check_case(resolver, ctx, id, expr),
        Labeled::Default(_) => check_default(resolver, id),
    };
    if let Err(diag) = res {
        resolver.sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_case(resolver: &mut SymbolResolver, ctx: &Context, id: StatementId, expr: &ExpressionNode) -> R {
    let value = resolver.eval_constant(ctx, expr);
    let sema = &mut resolver.sema;
    let Some(StatementScope::Switch {
        stmt, cases, control, ..
    }) = resolver.stmt_scopes.nearest_switch()
    else {
        return Err(Diagnosis::OutsideSwitch("case"));
    };
    let stmt = *stmt;
    let Some(value) = value else { return Err(Diagnosis::Poisoned) };
    if value.get_integer_value().is_none() {
        return Err(Diagnosis::NonIntegerConstantExpression);
    }
    let ty = sema.types.get(control.id);
    let Some(value) = Fold::new(&sema.target).convert(ty, value) else {
        return Err(Diagnosis::Poisoned);
    };
    if cases.iter().any(|(v, _)| value == *v) {
        return Err(Diagnosis::DuplicateCase(value));
    }
    cases.push((value, id));
    sema.stmts.set(id, Some(ResolvedStatement::Case(value, stmt)));
    Ok(())
}

fn check_default(resolver: &mut SymbolResolver, id: StatementId) -> R {
    let Some(StatementScope::Switch { stmt, default, .. }) = resolver.stmt_scopes.nearest_switch() else {
        return Err(Diagnosis::OutsideSwitch("default"));
    };
    let stmt = *stmt;
    if default.is_some() {
        return Err(Diagnosis::DuplicateDefault);
    }
    *default = Some(id);
    resolver.sema.stmts.set(id, Some(ResolvedStatement::Default(stmt)));
    Ok(())
}

pub fn enter_compound_statement(resolver: &mut SymbolResolver) {
    match resolver.scope_kind() {
        ScopeKind::Prototype => resolver.promote_scope(ScopeKind::Function),
        _ => resolver.enter_scope(ScopeKind::Block),
    }
}

pub fn leave_compound_statement(resolver: &mut SymbolResolver) {
    resolver.leave_scope();
}

pub fn check_selection_statement(sema: &mut Sema, node: &SelectionStatementNode) -> QualifiedType {
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

pub fn check_iteration_statement(sema: &mut Sema, node: &IterationStatementNode) {
    let res = match &node.stmt {
        IterationStatement::While(e, _) => check_scalar(sema, e),
        IterationStatement::Do(_, e) => check_scalar(sema, e),
        IterationStatement::For(b) => check_for(sema, b),
    };
    if let Err(diag) = res {
        sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_scalar(sema: &mut Sema, node: &ExpressionNode) -> Result<QualifiedType, Diagnosis> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &node.span);
    if !re.ty.is_scalar(sema) {
        return Err(Diagnosis::NonScalarStatement(re.ty));
    }
    Ok(re.ty)
}

fn check_integral(sema: &mut Sema, node: &ExpressionNode) -> Result<QualifiedType, Diagnosis> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &node.span);
    if !re.ty.is_integral(sema) {
        return Err(Diagnosis::NonIntegralStatement(re.ty));
    }
    cast::promote(sema, re);
    Ok(re.casted_ty())
}

fn check_for(
    sema: &mut Sema,
    b: &(
        ExpressionStatementNode,
        ExpressionStatementNode,
        Option<ExpressionNode>,
        StatementNode,
    ),
) -> Result<QualifiedType, Diagnosis> {
    let (e1, e2, e3, _) = b;
    e1.expr.as_ref().and_then(|e| expr_to_void(sema, e));
    e3.as_ref().and_then(|e| expr_to_void(sema, e));
    if let Some(e) = e2.expr.as_ref() {
        check_scalar(sema, e)
    } else {
        Ok(QualifiedType::plain(sema.builtins.int))
    }
}

pub fn check_jump_statement(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    id: StatementId,
    node: &JumpStatementNode,
    return_ty: Option<QualifiedType>,
) {
    let res = match &node.stmt {
        JumpStatement::Goto(name) => {
            resolver.reference_label(*name);
            resolver.sema.stmts.set(id, Some(ResolvedStatement::Goto(name.id)));
            Ok(())
        }
        JumpStatement::Return(_) => {
            check_return(resolver.sema, ctx, node, return_ty.expect("a return inside a function"))
        }
        JumpStatement::Break => check_break(resolver, id),
        JumpStatement::Continue => check_continue(resolver, id),
    };
    if let Err(diag) = res {
        resolver.sema.add_diag(Diag::err((), diag), &node.span);
    }
}

fn check_break(resolver: &mut SymbolResolver, id: StatementId) -> R {
    let &stmt = match resolver.stmt_scopes.last() {
        Some(StatementScope::Loop(stmt)) => stmt,
        Some(StatementScope::Switch { stmt, .. }) => stmt,
        None => return Err(Diagnosis::BreakNotInLoop),
    };
    resolver.sema.stmts.set(id, Some(ResolvedStatement::Break(stmt)));
    Ok(())
}

fn check_continue(resolver: &mut SymbolResolver, id: StatementId) -> R {
    let Some(stmt) = resolver.stmt_scopes.nearest_loop() else {
        return Err(Diagnosis::ContinueNotInLoop);
    };
    resolver.sema.stmts.set(id, Some(ResolvedStatement::Continue(stmt)));
    Ok(())
}

fn check_return(sema: &mut Sema, ctx: &Context, node: &JumpStatementNode, return_ty: QualifiedType) -> R {
    match &node.stmt {
        JumpStatement::Return(Some(e)) => {
            expression::init(sema, ctx, return_ty, e, AssignmentContext::Return).map(|_| ())
        }
        JumpStatement::Return(None) if return_ty.id != sema.builtins.void => Err(Diagnosis::ReturnWithoutValue),
        _ => Ok(()),
    }
}
