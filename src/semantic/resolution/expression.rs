use std::iter::zip;

use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, Expression, ExpressionNode, MemberOp, Name, Storage, Tag, Type, UnaryOp, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::ice::try_fold;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, ExpressionKind, ParamTypes, QualifiedType, ResolvedExpression,
    ResolvedType, Sema, SymbolKind, cast, declaration,
};

type R = Result<(QualifiedType, ExpressionKind), Diagnosis>;

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

pub fn init(
    sema: &mut Sema,
    ctx: &Context,
    l_ty: QualifiedType,
    init_node: &ExpressionNode,
    assign_ctx: AssignmentContext,
) -> R {
    let is_null = is_null_pointer_constant(sema, ctx, init_node);
    let mut ops = Operands::take(sema, [init_node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &init_node.span);
    let mut l_re = ResolvedExpression::new(l_ty, ExpressionKind::LValue);
    let out = cast::assignment_conversion(sema, &mut l_re, re, is_null, assign_ctx);
    if matches!(assign_ctx, AssignmentContext::Return)
        && let Some(cast) = re.casts.last_mut()
    {
        cast.to.is_volatile = false;
        cast.to.is_const = false;
    }
    out.map(|q| (q, RValue))
}

pub fn strip_qualifiers(sema: &mut Sema, node: &ExpressionNode) -> Result<(), Diagnosis> {
    let mut ops = Operands::take(sema, [node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &node.span);
    if let Some(cast) = re.casts.last_mut() {
        cast.to.is_volatile = false;
        cast.to.is_const = false;
    }
    Ok(())
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

fn is_null_pointer_constant(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> bool {
    let mut node = node;
    if let Expression::Cast(ty_node, op) = node.id.resolve(ctx) {
        let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
        let Some((qualif, _)) = declaration::declared_type(sema, ctx, base, &ty_node.declarator) else {
            return false;
        };
        if !matches!(qualif.id.resolve(sema), ResolvedType::Pointer(q) if q.is_void(sema))
            || qualif.is_const
            || qualif.is_volatile
        {
            return false;
        }
        node = op;
    }
    let Some(re) = sema.expr_resolved(node.id) else { return false };
    re.casted_ty().is_integer(sema) && try_fold(sema, ctx, node).is_some_and(|v| v.is_zero())
}

fn identifier(sema: &mut Sema, node: &ExpressionNode) -> R {
    let id = sema.binding(node.id).ok_poisoned()?;
    let sym = id.resolve(sema);
    Ok((sym.ty.ok_poisoned()?, sym.expression_kind()))
}

fn constant(sema: &mut Sema, e: &ExpressionNode) -> R {
    sema.expr_resolved(e.id)
        .map(|re| (re.casted_ty(), re.kind))
        .ok_poisoned()
}

fn array_subscript(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let (qty, _) = binary_op(sema, ctx, BinaryOp::Add, e1, e2)?;
    let ResolvedType::Pointer(inner) = qty.id.resolve(sema) else {
        return Err(Diagnosis::SubscriptNotArray);
    };
    Ok((*inner, LValue))
}
fn fn_call(sema: &mut Sema, ctx: &Context, fn_node: &ExpressionNode, args: &[ExpressionNode]) -> R {
    let mut ops = Operands::take(sema, [fn_node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &fn_node.span);
    let ty = re.casted_ty();
    let ResolvedType::Pointer(inner) = ty.id.resolve(sema) else {
        return Err(Diagnosis::CallingNotFunction(ty));
    };
    let ResolvedType::Function { ret, params } = inner.id.resolve(sema).clone() else {
        return Err(Diagnosis::CallingNotFunction(ty));
    };
    let returned = ret.id.resolve(sema);
    if !returned.is_void() && !returned.is_complete(sema) {
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
    for arg_node in args.iter().skip(param_len) {
        let promoted = Operands::take(&mut *sema, [arg_node]).map(|mut a| {
            let (sema, [re]) = a.parts();
            cast::lvalue_conversion(sema, re, &arg_node.span);
            cast::default_argument_promotions(sema, re);
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

fn member(sema: &mut Sema, node: &ExpressionNode, op: MemberOp, e: &ExpressionNode, name: &Name) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    let (&tag_qty, kind) = match op {
        MemberOp::Dot => (&re.casted_ty(), re.kind),
        MemberOp::Arrow => {
            cast::lvalue_conversion(sema, re, &e.span);
            let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve(sema) else {
                return Err(Diagnosis::AccessNotPointer(re.casted_ty()));
            };
            (inner, LValue)
        }
    };
    let &ResolvedType::Tag(tag_id) = tag_qty.id.resolve(sema) else {
        return Err(Diagnosis::AccessNotStuctOrUnion(tag_qty));
    };
    let tag = tag_id.resolve(sema);
    if !tag.is_complete {
        return Err(Diagnosis::IncompleteType(tag_qty));
    }
    if tag.kind != Tag::Struct && tag.kind != Tag::Union {
        return Err(Diagnosis::AccessNotStuctOrUnion(tag_qty));
    }
    let Some(sym_id) = tag.get_member(sema, name) else {
        return Err(Diagnosis::AccessNotMember(tag_qty, name.id));
    };
    sema.set_binding(node.id, Some(sym_id));
    let sym = sym_id.resolve(sema);
    let ty = sym.ty.ok_or(Diagnosis::Poisoned)?;
    let qty = QualifiedType::new(
        ty.id,
        ty.is_const || tag_qty.is_const,
        ty.is_volatile || tag_qty.is_volatile,
    );
    Ok((qty, kind))
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

fn inc_dec(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    check_assign_lhs(re)?;
    if let ResolvedType::Pointer(inner) = re.ty.id.resolve(sema)
        && !inner.is_complete(sema)
    {
        return Err(Diagnosis::IncompleteType(*inner));
    }
    if !re.ty.is_scalar(sema) {
        return Err(Diagnosis::BadPostIncDec(op, re.ty));
    }
    Ok((re.casted_ty(), RValue))
}

fn sign(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_arithmetic(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    cast::promote(sema, re);
    Ok((re.casted_ty(), RValue))
}

fn address(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    if re.kind == RValue && !re.casted_ty().is_function(sema) {
        return Err(Diagnosis::RValueAddress(re.ty));
    }
    if let Some(id) = sema.binding(e.id) {
        let sym = id.resolve(sema);
        if sym.storage == Some(Storage::Register) {
            return Err(Diagnosis::RegisterAddress);
        }
        if sym.kind == SymbolKind::Member && sym.value.is_some() {
            return Err(Diagnosis::BitFieldAddress);
        }
    }
    let ty = sema.types.pointer(re.ty);
    let qty = QualifiedType::new(ty, false, false);
    Ok((qty, RValue))
}

fn indirection(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::IndirectionNotPointer(re.ty));
    };
    if inner.is_void(sema) {
        return Err(Diagnosis::IncompleteType(*inner));
    }
    let kind = if inner.is_object(sema) { LValue } else { RValue };
    Ok((*inner, kind))
}

fn bit_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_integral(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    cast::promote(sema, re);
    Ok((re.casted_ty(), RValue))
}

fn logic_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    let qty = QualifiedType::new(sema.builtins.int, false, false);
    Ok((qty, RValue))
}

fn size_of_e(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    size_t(sema, re.ty)
}

fn size_of_ty(sema: &mut Sema, ctx: &Context, ty: &Type, span: &Span) -> R {
    let base = declaration::base_type(sema, ctx, &ty.specifiers, span);
    let (ty, _) = declaration::declared_type(sema, ctx, base, &ty.declarator).ok_poisoned()?;
    size_t(sema, ty)
}

fn size_t(sema: &Sema, ty: QualifiedType) -> R {
    if ty.is_void(sema) {
        return Err(Diagnosis::SizeofVoid);
    }
    if ty.is_function(sema) {
        return Err(Diagnosis::SizeofFunction);
    }
    if !ty.is_complete(sema) {
        return Err(Diagnosis::SizeofIncomplete(ty));
    }
    let qty = QualifiedType::new(sema.builtins.size_t, false, false);
    Ok((qty, RValue))
}

fn binary_op(sema: &mut Sema, ctx: &Context, op: BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    match op {
        BinaryOp::Add | BinaryOp::Sub => additive(sema, e1, e2, op),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative(sema, e1, e2, op),
        BinaryOp::Right | BinaryOp::Left => shift(sema, ctx, e1, e2),
        BinaryOp::Greater | BinaryOp::Lower | BinaryOp::GreaterEq | BinaryOp::LowerEq => relational(sema, e1, e2),
        BinaryOp::Eq | BinaryOp::Neq => equality(sema, ctx, e1, e2),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise(sema, e1, e2),
        _ => todo!(),
    }
}

fn multiplicative_types(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    op: BinaryOp,
) -> R {
    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    if !match op {
        BinaryOp::Mul | BinaryOp::Div => l.is_arithmetic(sema) && r.is_arithmetic(sema),
        BinaryOp::Mod => l.is_integral(sema) && r.is_integral(sema),
        _ => unreachable!(),
    } {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.casted_ty(), rhs.casted_ty()));
    }
    cast::usual_arithmetic(sema, lhs, rhs)
}

fn multiplicative(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode, op: BinaryOp) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    multiplicative_types(sema, lhs, rhs, op)
}

fn additive_types(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression, op: BinaryOp) -> R {
    match (op, lhs.casted_ty().id.resolve(sema), rhs.casted_ty().id.resolve(sema)) {
        (_, l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => cast::usual_arithmetic(sema, lhs, rhs),
        (_, ResolvedType::Pointer(_), o) if o.is_integral(sema) => cast::pointer_integer_arithmetic(sema, lhs, rhs),
        (BinaryOp::Add, o, ResolvedType::Pointer(_)) if o.is_integral(sema) => {
            cast::pointer_integer_arithmetic(sema, rhs, lhs)
        }
        (BinaryOp::Sub, ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => {
            cast::pointer_minus_pointer(sema, lhs, rhs)
        }
        _ => Err(Diagnosis::InvalidOperand),
    }
}

fn additive(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode, op: BinaryOp) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    additive_types(sema, lhs, rhs, op)
}

fn shift_types(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression, count: Option<Value>) -> R {
    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    if !l.is_integral(sema) || !r.is_integral(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.casted_ty(), rhs.casted_ty()));
    }
    cast::promote(sema, lhs);
    cast::promote(sema, rhs);
    let l = lhs.casted_ty().id.resolve(sema);
    let l_layout = sema.target.layout(l).unwrap();
    let Some(count) = count else { return Ok((lhs.casted_ty(), RValue)) };
    if count.is_negative() {
        return Err(Diagnosis::ShiftCountNegative);
    }
    if count.is_greater_or_eq(l_layout.size * sema.target.byte_size) {
        return Err(Diagnosis::ShiftCountOutOfRange);
    }
    Ok((lhs.casted_ty(), RValue))
}

fn shift(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let count = try_fold(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    shift_types(sema, lhs, rhs, count)
}

fn relational(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    let ret = Ok((ty, RValue));
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::usual_arithmetic(sema, lhs, rhs)?;
        return ret;
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    };
    if !i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }

    if i1.is_object(sema) && i2.is_object(sema) {
        return ret;
    }
    if i1.is_complete(sema) && i2.is_complete(sema) {
        return ret;
    }
    Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty))
}

fn equality(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let null1 = is_null_pointer_constant(sema, ctx, e1);
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    let ret = Ok((ty, RValue));
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::usual_arithmetic(sema, lhs, rhs)?;
        return ret;
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    };
    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return ret;
    }
    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, lhs.casted_ty().id, false);
        return ret;
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, rhs.casted_ty().id, false);
        return ret;
    }

    if null1 || null2 {
        return ret;
    }
    Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty))
}

fn bitwise(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    if !lhs.ty.is_integral(sema) || !rhs.ty.is_integral(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    cast::usual_arithmetic(sema, lhs, rhs)?;
    Ok((lhs.casted_ty(), RValue))
}

fn cast(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, ty_node: &Type, operand: &ExpressionNode) -> R {
    let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(sema, ctx, operand);
    let mut ops = Operands::take(sema, [operand])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &operand.span);
    let ty = qualif.id.resolve(sema);
    if !ty.is_void() {
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
}

fn assign(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let is_null = is_null_pointer_constant(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    check_assign_lhs(lhs)?;
    cast::lvalue_conversion(sema, rhs, &e2.span);
    let q = cast::assignment_conversion(sema, lhs, rhs, is_null, AssignmentContext::Assignment)?;
    Ok((q, ExpressionKind::RValue))
}

fn list(sema: &mut Sema, es: &[ExpressionNode]) -> R {
    for node in es[0..(es.len() - 1)].iter() {
        let Ok(mut ops) = Operands::take(&mut *sema, [node]) else { continue };
        let (sema, [re]) = ops.parts();
        cast::lvalue_conversion(sema, re, &node.span);
        cast::to_void(sema, re);
    }
    let last = es.last().unwrap();
    let mut ops = Operands::take(sema, [last])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &last.span);
    Ok((re.casted_ty(), RValue))
}

fn type_of(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> R {
    use ExpressionKind::{LValue, RValue};
    match node.id.resolve(ctx) {
        Expression::Identifier(_) => identifier(sema, node),
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), LValue)),
        Expression::ConstantExpression(e) => constant(sema, e),
        Expression::ArraySubscripting(e1, e2) => array_subscript(sema, ctx, e1, e2),
        Expression::FunctionCall(fn_node, args) => fn_call(sema, ctx, fn_node, args),
        Expression::Member(op, e, name) => member(sema, node, *op, e, name),
        Expression::Unary(op, e) => unary_op(sema, *op, e),
        Expression::SizeofExpr(e) => size_of_e(sema, e),
        Expression::SizeofType(ty) => size_of_ty(sema, ctx, ty, &node.span),
        Expression::Binary(op, e1, e2) => binary_op(sema, ctx, *op, e1, e2),
        Expression::Cast(ty_node, operand) => cast(sema, ctx, node, ty_node, operand),
        Expression::Assign(None, e1, e2) => assign(sema, ctx, e1, e2),
        Expression::List(es) => list(sema, es),
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

struct Operands<'s, 'n, const N: usize> {
    sema: &'s mut Sema,
    nodes: [&'n ExpressionNode; N],
    res: Option<[ResolvedExpression; N]>,
}

impl<'s, 'n, const N: usize> Operands<'s, 'n, N> {
    fn take(sema: &'s mut Sema, nodes: [&'n ExpressionNode; N]) -> Result<Self, Diagnosis> {
        let mut taken = std::array::from_fn(|i| sema.take_expr_resolved(nodes[i].id));
        if taken.iter().any(Option::is_none) {
            for (node, re) in zip(nodes, &mut taken) {
                if let Some(re) = re.take() {
                    sema.set_expr_resolved(node.id, Some(re));
                }
            }
            return Err(Diagnosis::Poisoned);
        }
        Ok(Self {
            sema,
            nodes,
            res: Some(taken.map(Option::unwrap)),
        })
    }

    fn parts(&mut self) -> (&mut Sema, &mut [ResolvedExpression; N]) {
        (self.sema, self.res.as_mut().unwrap())
    }
}

impl<const N: usize> Drop for Operands<'_, '_, N> {
    fn drop(&mut self) {
        for (node, re) in zip(self.nodes, self.res.take().unwrap()) {
            self.sema.set_expr_resolved(node.id, Some(re));
        }
    }
}
