use crate::arena::{Loan, OptionPoisoned};
use crate::ast::Statement;
use crate::ast::{
    ExpressionNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, SelectionStatement, SelectionStatementNode, StatementNode, statement::StatementId,
};
use crate::context::Context;
use crate::semantic::resolution::expression::{self, operands};
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, QualifiedType, ResolvedStatement, Sema, cast,
};

type Substatements<'s, const N: usize> = Loan<'s, Sema, StatementId, ResolvedStatement, N>;

fn substatements<'s, const N: usize>(
    sema: &'s mut Sema,
    nodes: [&StatementNode; N],
) -> Result<Substatements<'s, N>, Diagnosis> {
    Loan::take(sema, nodes.map(|n| n.id)).ok_poisoned()
}

// pub fn st(sema: &mut Sema, ctx: &Context, stmt: StatementNode) {
//     match stmt.id.resolve(ctx) {
//         Statement::Labeled(l) => (),
//     }
// }

pub fn check_selection_statement(sema: &mut Sema, node: &SelectionStatementNode) {
    let res = match &node.stmt {
        SelectionStatement::If(e1, _, _) => check_scalar(sema, e1),
        SelectionStatement::Switch(e, _) => check_integral(sema, e),
    };
    if let Err(diag) = res {
        sema.add_diag(Diag::err((), diag), &node.span);
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

fn check_scalar(sema: &mut Sema, node: &ExpressionNode) -> Result<(), Diagnosis> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &node.span);
    if !re.ty.is_scalar(sema) {
        return Err(Diagnosis::NonScalarStatement(re.ty));
    }
    Ok(())
}

fn check_integral(sema: &mut Sema, node: &ExpressionNode) -> Result<(), Diagnosis> {
    let mut ops = operands(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &node.span);
    if !re.ty.is_integral(sema) {
        return Err(Diagnosis::NonIntegralStatement(re.ty));
    }
    Ok(())
}

fn check_for(
    sema: &mut Sema,
    b: &(
        ExpressionStatementNode,
        ExpressionStatementNode,
        Option<ExpressionNode>,
        StatementNode,
    ),
) -> Result<(), Diagnosis> {
    let (e1, e2, e3, _) = b;
    e1.expr.as_ref().and_then(|e| expr_to_void(sema, e));
    e3.as_ref().and_then(|e| expr_to_void(sema, e));
    if let Some(e) = e2.expr.as_ref() { check_scalar(sema, e) } else { Ok(()) }
}

pub fn check_return(sema: &mut Sema, ctx: &Context, node: &JumpStatementNode, return_ty: QualifiedType) {
    match &node.stmt {
        JumpStatement::Return(Some(e)) => {
            if let Err(inner) = expression::init(sema, ctx, return_ty, e, AssignmentContext::Return) {
                sema.add_diag(Diag::err((), inner), &e.span);
            }
        }
        JumpStatement::Return(None) if return_ty.id != sema.builtins.void => {
            sema.add_diag(Diag::err((), Diagnosis::InvalidReturnType), &node.span);
        }
        _ => (),
    }
}
