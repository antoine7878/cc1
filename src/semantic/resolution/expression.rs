use std::iter::zip;

use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, Expression, ExpressionNode, MemberOp, Name, Tag, Type, UnaryOp};
use crate::context::Context;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::ice::try_fold;
use crate::semantic::model::cast;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, ExpressionKind, ParamTypes, QualifiedType, ResolvedExpression,
    ResolvedType, ResolvedTypeId, Sema, declaration,
};

pub fn resolve_expression(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) {
    if sema.expr_seen(node.id) {
        return;
    }
    let resolved = match type_of(sema, ctx, node) {
        Ok((ty, kind)) => Some(ResolvedExpression::new(ty, kind)),
        Err(inner) => {
            sema.add_diag(Diag::err((), inner), &node.span);
            None
        }
    };
    sema.set_expr_resolved(node.id, resolved);
}

fn take(sema: &mut Sema, node: &ExpressionNode) -> Result<ResolvedExpression, Diagnosis> {
    sema.take_expr_resolved(node.id).ok_poisoned()
}

fn takes(
    sema: &mut Sema,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> Result<(ResolvedExpression, ResolvedExpression), Diagnosis> {
    let lhs = take(sema, e1)?;
    let rhs = match take(sema, e2) {
        Ok(rhs) => rhs,
        Err(diag) => {
            put(sema, e1, lhs);
            return Err(diag);
        }
    };
    Ok((lhs, rhs))
}

fn put(sema: &mut Sema, node: &ExpressionNode, re: ResolvedExpression) {
    sema.set_expr_resolved(node.id, Some(re));
}

fn puts(
    sema: &mut Sema,
    (e1, re1): (&ExpressionNode, ResolvedExpression),
    (e2, re2): (&ExpressionNode, ResolvedExpression),
) {
    put(sema, e1, re1);
    put(sema, e2, re2);
}

pub fn init(
    sema: &mut Sema,
    ctx: &Context,
    l_ty: QualifiedType,
    init_node: &ExpressionNode,
    assign_ctx: AssignmentContext,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    let is_null = is_null_pointer_constant(sema, ctx, init_node);
    with_operand(sema, init_node, |sema, re| {
        let mut l_re = ResolvedExpression::new(l_ty, ExpressionKind::LValue);
        let out = cast::assignment_conversion(sema, &mut l_re, re, is_null, assign_ctx);
        if matches!(assign_ctx, AssignmentContext::Return)
            && let Some(cast) = re.casts.last_mut()
        {
            cast.to.is_volatile = false;
            cast.to.is_const = false;
        }
        out.map(|q| (q, RValue))
    })
}

pub fn strip_qualifiers(sema: &mut Sema, node: &ExpressionNode) -> Result<(), Diagnosis> {
    with_operand(sema, node, |_sema, re| {
        if let Some(cast) = re.casts.last_mut() {
            cast.to.is_volatile = false;
            cast.to.is_const = false;
        }
        Ok((re.casted_ty(), RValue))
    })
    .map(|_| ())
}

fn with_operand<F>(sema: &mut Sema, node: &ExpressionNode, f: F) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression) -> Result<(QualifiedType, ExpressionKind), Diagnosis>,
{
    let mut re = take(sema, node)?;
    cast::lvalue_conversion(sema, &mut re, &node.span);
    let out = f(sema, &mut re);
    put(sema, node, re);
    out
    // out.map(|q| (q, kind))
}

fn with_operands<F>(
    sema: &mut Sema,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    kind: ExpressionKind,
    f: F,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let (mut lhs, mut rhs) = takes(sema, e1, e2)?;
    cast::lvalue_conversion(sema, &mut lhs, &e1.span);
    cast::lvalue_conversion(sema, &mut rhs, &e2.span);
    let out = f(sema, &mut lhs, &mut rhs);
    puts(sema, (e1, lhs), (e2, rhs));
    out.map(|q| (q, kind))
}

fn check_assign_lhs(lhs: &ResolvedExpression) -> Result<(), Diagnosis> {
    if lhs.kind == ExpressionKind::RValue {
        return Err(Diagnosis::AssignToRValue);
    }
    if lhs.ty.is_const {
        return Err(Diagnosis::ConstAssignment(lhs.ty));
    }
    Ok(())
}

fn with_assignment<F>(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    f: F,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression, bool) -> Result<QualifiedType, Diagnosis>,
{
    let is_null = is_null_pointer_constant(sema, ctx, e2);
    let (mut lhs, mut rhs) = takes(sema, e1, e2)?;
    let out = check_assign_lhs(&lhs).and_then(|()| {
        cast::lvalue_conversion(sema, &mut rhs, &e2.span);
        f(sema, &mut lhs, &mut rhs, is_null)
    });
    puts(sema, (e1, lhs), (e2, rhs));
    out.map(|q| (q, ExpressionKind::RValue))
}

fn as_written<'a>(sema: &'a Sema, node: &ExpressionNode) -> Result<&'a ResolvedExpression, Diagnosis> {
    sema.expr_resolved(node.id).ok_poisoned()
}

fn check_is_arithmetic(sema: &Sema, ty: ResolvedTypeId) -> Result<(), Diagnosis> {
    match ty.resolve(sema).is_arithmetic(sema) {
        false => Err(Diagnosis::InvalidOperand),
        true => Ok(()),
    }
}

fn is_null_pointer_constant(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> bool {
    let mut node = node;
    if let Expression::Cast(ty_node, op) = node.id.resolve(ctx) {
        let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
        let Some((qualif, _)) = declaration::declared_type(sema, ctx, base, &ty_node.declarator) else {
            return false;
        };
        if !matches!(qualif.id.resolve(sema), ResolvedType::Pointer(q) if q.id == sema.builtins.void)
            || qualif.is_const
            || qualif.is_volatile
        {
            return false;
        }
        node = op;
    }
    let Some(re) = sema.expr_resolved(node.id) else { return false };
    re.casted_ty().id.resolve(sema).is_integer() && try_fold(sema, ctx, node).is_some_and(|v| v.is_zero())
}

fn check_fn_call(
    sema: &mut Sema,
    ctx: &Context,
    ty: QualifiedType,
    args: &[ExpressionNode],
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    let ResolvedType::Pointer(inner) = ty.id.resolve(sema) else {
        return Err(Diagnosis::CallingNotFunction(ty));
    };
    let ResolvedType::Function { ret, params } = inner.id.resolve(sema).clone() else {
        return Err(Diagnosis::CallingNotFunction(ty));
    };
    // 6.3.2.2 The expression that denotes the called function shall have type pointer to function
    // returning void or returning an object type other than an array type.
    let returned = ret.id.resolve(sema);
    if !matches!(returned, ResolvedType::Void) && !returned.is_complete(sema) {
        return Err(Diagnosis::CallingIncompleteReturn(ret));
    }
    let (params, is_variadic) = match params {
        ParamTypes::Prototype { params, is_variadic } => (params, is_variadic),
        ParamTypes::Unspecified => (vec![], true),
    };
    if !is_variadic && args.len() > params.len() {
        return Err(Diagnosis::TooManyArguments(params.len(), args.len()));
    }
    if args.len() < params.len() {
        return Err(Diagnosis::TooFewArguments(params.len(), args.len()));
    }
    let param_len = params.len();
    let mut rejected = false;
    for (n, (param_ty, arg_node)) in zip(params, args).enumerate() {
        if let Err(err) = init(sema, ctx, param_ty, arg_node, AssignmentContext::Argument(n + 1)) {
            sema.add_diag(Diag::err((), err), &arg_node.span);
            rejected = true;
        }
    }
    // 6.3.2.2 The default argument promotions are performed on trailing arguments.
    for arg_node in args.iter().skip(param_len) {
        let promoted = with_operand(sema, arg_node, |sema, re| {
            cast::default_argument_promotions(sema, re);
            Ok((re.casted_ty(), RValue))
        });
        if let Err(err) = promoted {
            sema.add_diag(Diag::err((), err), &arg_node.span);
            rejected = true;
        }
    }
    match rejected {
        true => Err(Diagnosis::Poisoned),
        false => Ok((QualifiedType::new(ret.id, false, false), RValue)),
    }
}

fn member(
    sema: &mut Sema,
    op: MemberOp,
    e: &ExpressionNode,
    name: &Name,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    with_operand(sema, e, |sema, re| {
        let (tag_qty, kind) = match op {
            MemberOp::Dot => (&re.casted_ty(), re.kind),
            MemberOp::Arrow => {
                let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve(sema) else {
                    return Err(Diagnosis::AccessNotPointer(re.casted_ty()));
                };
                (inner, LValue)
            }
        };
        let &ResolvedType::Tag(tag_id) = tag_qty.id.resolve(sema) else {
            return Err(Diagnosis::AccessNotStuctOrUnion(re.casted_ty()));
        };
        let tag = tag_id.resolve(sema);
        if tag.kind != Tag::Struct && tag.kind != Tag::Union {
            return Err(Diagnosis::AccessNotStuctOrUnion(re.casted_ty()));
        }
        let Some(sym_id) = tag.get_member(sema, name) else {
            return Err(Diagnosis::AccessNotMember(re.ty, name.id));
        };
        let sym = sym_id.resolve(sema);
        let ty = sym.ty.ok_or(Diagnosis::Poisoned)?;
        Ok((ty, kind))
    })
}

fn unary_op(sema: &mut Sema, op: UnaryOp, e: &ExpressionNode) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    match op {
        UnaryOp::PostInc | UnaryOp::PostDec => postfix(sema, e, op),
        _ => todo!(),
    }
}

fn postfix(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    with_operand(sema, e, |sema, re| {
        let q = re.casted_ty();
        if re.casted_ty().is_const {
            return Err(Diagnosis::ConstAssignment(re.ty));
        }
        if !re.casted_ty().id.resolve(sema).is_scalar(sema) {
            return Err(Diagnosis::BadPostIncDec(op, q));
        }
        Ok((re.casted_ty(), RValue))
    })
}

fn binary_op(
    sema: &mut Sema,
    ctx: &Context,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    match op {
        BinaryOp::Add | BinaryOp::Sub => additive(sema, e1, e2, op),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative(sema, ctx, e1, e2, op),
        BinaryOp::Right | BinaryOp::Left => shift(sema, ctx, e1, e2),
        _ => todo!(),
    }
}

// 6.3.5 Multiplicative operators
// Each of the operands shall have arithmetic type. The operands of the % operator shall have integral type.
// The usual arithmetic conversions are performed on the operands.
// The result of the / operator is the quotient from the division of the first operand by the
// second; the result of the % operator is the remainder. In both operations. if the value of the
// second operand is zero. the behavior is undefined.
fn multiplicative(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    op: BinaryOp,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    if try_fold(sema, ctx, e2).is_some_and(|v| v.is_zero()) {
        match op {
            BinaryOp::Mod => return Err(Diagnosis::ModuloByZero),
            BinaryOp::Div => return Err(Diagnosis::DivisionByZero),
            _ => (),
        }
    }
    with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| {
        let l = lhs.casted_ty().id.resolve(sema);
        let r = rhs.casted_ty().id.resolve(sema);
        if !match op {
            BinaryOp::Mod => l.is_integral(sema) || r.is_integral(sema),
            BinaryOp::Mul | BinaryOp::Div => l.is_arithmetic(sema) || r.is_arithmetic(sema),
            _ => panic!("non multiplicative operator"),
        } {
            return Err(Diagnosis::InvalidBianryOperand(lhs.casted_ty(), rhs.casted_ty()));
        }
        cast::usual_arithmetic(sema, lhs, rhs);
        Ok(lhs.casted_ty())
    })
}

// 6.3.6 Additive operators
// For addition. either both operands shall have arithmetic type. or one operand shall be a
// pointer to an object type and the other shall have integral type.
// For subtraction. one of the following shall hold:
// - both operands have arithmetic type;
// - both operands are pointers to qualified or unqualitied versions of compatible object types
// - the left operand is a pointer to an object type and the right operand has integral type
fn additive(
    sema: &mut Sema,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    op: BinaryOp,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| {
        match (op, lhs.casted_ty().id.resolve(sema), rhs.casted_ty().id.resolve(sema)) {
            (_, l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => {
                cast::usual_arithmetic(sema, lhs, rhs);
                Ok(lhs.casted_ty())
            }
            (_, ResolvedType::Pointer(_), o) if o.is_integral(sema) => cast::pointer_integer_arithmetic(sema, lhs, rhs),
            (BinaryOp::Add, o, ResolvedType::Pointer(_)) if o.is_integral(sema) => {
                cast::pointer_integer_arithmetic(sema, rhs, lhs)
            }
            (BinaryOp::Sub, ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => {
                cast::pointer_minus_pointer(sema, lhs, rhs)
            }
            _ => Err(Diagnosis::InvalidOperand),
        }
    })
}

// 6.3.7 Bitwise shift operators
// The integral promotions are performed on each of the operands. The type of the result is that
// of the promoted left operand. If the value of the right operand is negative or is greater than or
// equal to the width in bits of the promoted left operand. the behavior is undefined.
fn shift(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    let v = try_fold(sema, ctx, e2);
    with_operands(sema, e1, e2, RValue, |sema, rhs, lhs| {
        let l = lhs.casted_ty().id.resolve(sema);
        let r = rhs.casted_ty().id.resolve(sema);
        if !l.is_integral(sema) || !r.is_integral(sema) {
            return Err(Diagnosis::InvalidBianryOperand(lhs.casted_ty(), rhs.casted_ty()));
        }
        let r_layout = sema.target.layout(r).unwrap();
        cast::usual_arithmetic(sema, lhs, rhs);
        let Some(v) = v else { return Ok(lhs.casted_ty()) };
        if v.is_negative() {
            return Err(Diagnosis::ShiftCountNegative);
        }
        if v.is_greater_or_eq(r_layout.size) {
            return Err(Diagnosis::ShiftCountOutOfRange);
        }
        Ok(lhs.casted_ty())
    })
}

// // 6.3.8 Relational operators
// fn relational(
//     sema: &mut Sema,
//     ctx: &Context,
//     e1: &ExpressionNode,
//     e2: &ExpressionNode,
// ) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
//     with_operands(sema, e1, e2, RValue, |sema, rhs, lhs| {
//         let l = lhs.casted_ty().id.resolve(sema);
//         let r = rhs.casted_ty().id.resolve(sema);
//         if !l.is_integral(sema) || !r.is_integral(sema) {
//             return Err(Diagnosis::InvalidBianryOperand(lhs.casted_ty(), rhs.casted_ty()));
//         }
//     })
// }

fn cast(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
    ty_node: &Type,
    operand: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(sema, ctx, operand);
    with_operand(sema, operand, |sema, re| {
        let ty = qualif.id.resolve(sema);
        if !matches!(ty, ResolvedType::Void) {
            let from = re.casted_ty().id.resolve(sema);
            if !ty.is_scalar(sema) || !from.is_scalar(sema) {
                return Err(Diagnosis::CastToNonScalar);
            }
            if ty.is_pointer() != from.is_pointer() && (ty.is_floating() || from.is_floating()) {
                return Err(Diagnosis::InvalidOperand);
            }
            cast::convert(sema, re, qualif.id, is_null);
        }
        Ok((qualif, RValue))
    })
}

fn type_of(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    use ExpressionKind::{LValue, RValue};

    match node.id.resolve(ctx) {
        // 6.3.1 Primary expressions
        Expression::Identifier(_) => {
            let id = sema.binding(node.id).ok_poisoned()?;
            let sym = id.resolve(sema);
            Ok((sym.ty.ok_poisoned()?, sym.expression_kind()))
        }
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), LValue)),
        Expression::ConstantExpression(expr) => as_written(sema, expr).map(|re| (re.ty, re.kind)),
        // 6.3.2.1 Array subscripting
        Expression::ArraySubscripting(e1, e2) => binary_op(sema, ctx, BinaryOp::Add, e1, e2),
        // 6.3.2.2 Function calls
        Expression::FunctionCall(fn_node, args) => {
            with_operand(sema, fn_node, |sema, re| check_fn_call(sema, ctx, re.casted_ty(), args))
        }
        // 6.3.2.3 Structure and union members
        Expression::Member(op, e, name) => member(sema, *op, e, name),
        // 6.3.2.4 Postfix increment and decrement operators
        Expression::Unary(op, e) => unary_op(sema, *op, e),
        // Expression::Unary(UnaryOp::Minus, e) => with_operand(sema, e, RValue, |sema, re| {
        //     check_is_arithmetic(sema, re.casted_ty().id)?;
        //     cast::promote(sema, re);
        //     Ok(re.casted_ty())
        // }),
        Expression::Binary(op, e1, e2) => binary_op(sema, ctx, *op, e1, e2),
        Expression::Cast(ty_node, operand) => cast(sema, ctx, node, ty_node, operand),
        Expression::Assign(None, e1, e2) => with_assignment(sema, ctx, e1, e2, |sema, lhs, rhs, is_null| {
            cast::assignment_conversion(sema, lhs, rhs, is_null, AssignmentContext::Assignment)
        }),
        Expression::List(es) => {
            for node in es[0..(es.len() - 1)].iter() {
                let _ = with_operand(sema, node, |sema, re| {
                    cast::to_void(sema, re);
                    Ok((re.casted_ty(), RValue))
                });
            }
            with_operand(sema, es.last().unwrap(), |_, re| Ok((re.casted_ty(), RValue)))
        }
        _ => Err(Diagnosis::Poisoned),
    }
}

trait OptionPoisoned<T> {
    fn ok_poisoned(self) -> Result<T, Diagnosis>;
}

impl<T> OptionPoisoned<T> for Option<T> {
    fn ok_poisoned(self) -> Result<T, Diagnosis> {
        self.ok_or(Diagnosis::Poisoned)
    }
}
