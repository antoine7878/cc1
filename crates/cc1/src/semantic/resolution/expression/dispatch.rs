use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::semantic::ValueCategory::{LValue, RValue};
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diag, DiagnosticSink, ResolvedExpression, Resolver, Sema, cast};

pub fn resolve_expression(resolver: &mut Resolver, node: &ExpressionNode) {
    if resolver.sema.expressions.contains(node.id) {
        return;
    }
    let resolved = match type_of(resolver, node) {
        Ok((ty, kind)) => {
            let mut re = ResolvedExpression::new(ty, kind);
            re.bit_width = resolver.sema.member_refs.get(node.id).and_then(|r| r.member(resolver.sema).width);
            Some(re)
        }
        Err(diag) => {
            resolver.add_diag(Diag::err((), diag), &node.span);
            None
        }
    };
    resolver.sema.expressions.set(node.id, resolved);
}

fn type_of(resolver: &mut Resolver, node: &ExpressionNode) -> ExprResult {
    let sema = &mut *resolver.sema;
    match node.id.resolve() {
        Expression::Identifier(_) => identifier(sema, node),
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema), LValue)),
        Expression::ConstantExpression(e) => constant(sema, e),
        Expression::ArraySubscripting(e1, e2) => array_subscript(sema, e1, e2),
        Expression::FunctionCall(fn_node, args) => function_call(sema, fn_node, args),
        Expression::Member(op, e, name) => member(sema, node, *op, e, name),
        Expression::Unary(op, e) => unary_op(sema, *op, e),
        Expression::SizeofExpr(e) => sizeof_expr(sema, node, e),
        Expression::SizeofType(ty) => sizeof_type(resolver, node, ty, &node.span),
        Expression::Binary(op, e1, e2) => binary_op(sema, op, e1, e2),
        Expression::Ternary(e1, e2, e3) => conditional(sema, e1, e2, e3),
        Expression::Assign(op, e1, e2) => assignment(sema, op, e1, e2),
        Expression::Cast(ty_node, operand) => cast(resolver, node, ty_node, operand),
        Expression::List(es) => list(sema, es),
    }
}

pub fn binary_op(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    match op {
        BinaryOp::Add | BinaryOp::Sub => additive(sema, op, e1, e2),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative(sema, op, e1, e2),
        BinaryOp::Right | BinaryOp::Left => shift(sema, e1, e2),
        BinaryOp::Greater | BinaryOp::Lower | BinaryOp::GreaterEq | BinaryOp::LowerEq => relational(sema, e1, e2),
        BinaryOp::Eq | BinaryOp::Neq => equality(sema, e1, e2),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise(sema, e1, e2),
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => logical(sema, e1, e2),
    }
}

fn unary_op(sema: &mut Sema, op: UnaryOp, e: &ExpressionNode) -> ExprResult {
    match op {
        UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => inc_dec(sema, e, op),
        UnaryOp::Plus | UnaryOp::Minus => unary_sign(sema, e),
        UnaryOp::Addr => address(sema, e),
        UnaryOp::Deref => indirection(sema, e),
        UnaryOp::BitNot => bit_not(sema, e),
        UnaryOp::LogicalNot => logical_not(sema, e),
    }
}

fn assignment(sema: &mut Sema, op: &Option<BinaryOp>, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    match op {
        None => simple_assignment(sema, e1, e2),
        Some(BinaryOp::Add | BinaryOp::Sub) => additive_assignment(sema, e1, e2),
        Some(
            op @ (BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Left
            | BinaryOp::Right
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor),
        ) => compound_assignment(sema, op, e1, e2),
        _ => unreachable!(),
    }
}

fn list(sema: &mut Sema, es: &[ExpressionNode]) -> ExprResult {
    for node in es[0..(es.len() - 1)].iter() {
        let _ = with_converted(&mut *sema, [node], |sema, [re]| {
            cast::to_void(sema, re);
            Ok(())
        });
    }
    let last = es.last().unwrap();
    with_converted(sema, [last], |_, [re]| Ok((re.casted_ty(), RValue)))
}
