use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::context::Context;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::{Diag, DiagCollector, ResolvedExpression, Sema, cast};

use super::arithmetic::{additive, multiplicative, shift};
use super::assign::{additive_assignment, compound_assignment, simple_assignment};
use super::compare::{bitwise, equality, logic, relational};
use super::conditional::conditional;
use super::operand::{R, with_converted};
use super::primary::{array_subscript, constant, fn_call, identifier, member};
use super::sizeof::{size_of_e, size_of_ty};
use super::unary::{address, bit_not, cast, inc_dec, indirection, logic_not, sign};

pub fn resolve_expression(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) {
    if sema.expr_types.seen(node.id) {
        return;
    }
    let resolved = match type_of(sema, ctx, node) {
        Ok((ty, kind)) => Some(ResolvedExpression::new(ty, kind)),
        Err(diag) => {
            sema.add_diag(Diag::err((), diag), &node.span);
            None
        }
    };
    sema.expr_types.set(node.id, resolved);
}

fn type_of(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> R {
    match node.id.resolve(ctx) {
        Expression::Identifier(_) => identifier(sema, node),
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), LValue)),
        Expression::ConstantExpression(e) => constant(sema, e),
        Expression::ArraySubscripting(e1, e2) => array_subscript(sema, ctx, e1, e2),
        Expression::FunctionCall(fn_node, args) => fn_call(sema, ctx, fn_node, args),
        Expression::Member(op, e, name) => member(sema, node, *op, e, name),
        Expression::Unary(op, e) => unary_op(sema, *op, e),
        Expression::SizeofExpr(e) => size_of_e(sema, node, e),
        Expression::SizeofType(ty) => size_of_ty(sema, ctx, node, ty, &node.span),
        Expression::Binary(op, e1, e2) => binary_op(sema, ctx, op, e1, e2),
        Expression::Ternary(e1, e2, e3) => conditional(sema, ctx, e1, e2, e3),
        Expression::Assign(op, e1, e2) => assignment(sema, ctx, op, e1, e2),
        Expression::Cast(ty_node, operand) => cast(sema, ctx, node, ty_node, operand),
        Expression::List(es) => list(sema, es),
    }
}

pub fn binary_op(sema: &mut Sema, ctx: &Context, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    match op {
        BinaryOp::Add | BinaryOp::Sub => additive(sema, op, e1, e2),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative(sema, op, e1, e2),
        BinaryOp::Right | BinaryOp::Left => shift(sema, ctx, e1, e2),
        BinaryOp::Greater | BinaryOp::Lower | BinaryOp::GreaterEq | BinaryOp::LowerEq => relational(sema, e1, e2),
        BinaryOp::Eq | BinaryOp::Neq => equality(sema, ctx, e1, e2),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise(sema, e1, e2),
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => logic(sema, e1, e2),
    }
}

fn unary_op(sema: &mut Sema, op: UnaryOp, e: &ExpressionNode) -> R {
    match op {
        UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => inc_dec(sema, e, op),
        UnaryOp::Plus | UnaryOp::Minus => sign(sema, e),
        UnaryOp::Addr => address(sema, e),
        UnaryOp::Deref => indirection(sema, e),
        UnaryOp::BitNot => bit_not(sema, e),
        UnaryOp::LogicalNot => logic_not(sema, e),
    }
}

fn assignment(sema: &mut Sema, ctx: &Context, op: &Option<BinaryOp>, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    match op {
        None => simple_assignment(sema, ctx, e1, e2),
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
        ) => compound_assignment(sema, ctx, op, e1, e2),
        _ => unreachable!(),
    }
}

fn list(sema: &mut Sema, es: &[ExpressionNode]) -> R {
    for node in es[0..(es.len() - 1)].iter() {
        let _ = with_converted(&mut *sema, [node], |sema, [re]| {
            cast::to_void(sema, re);
            Ok(())
        });
    }
    let last = es.last().unwrap();
    with_converted(sema, [last], |_, [re]| Ok((re.casted_ty(), RValue)))
}
