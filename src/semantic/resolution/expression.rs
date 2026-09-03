use std::iter::zip;

use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, Expression, ExpressionNode, MemberOp, Name, Storage, Tag, Type, UnaryOp, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::ice::try_fold;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, ExpressionKind, ParamTypes, QualifiedType, ResolvedExpression,
    ResolvedType, Sema, SymbolId, SymbolKind, cast, declaration, layout,
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

fn check_assignable(lhs: &ResolvedExpression) -> Result<(), Diagnosis> {
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
        if let Some((qualif, _)) = declaration::declared_type(sema, ctx, base, &ty_node.declarator)
            && is_void_pointer(sema, qualif)
        {
            node = op;
        }
    }
    let Some(re) = sema.expr_resolved(node.id) else { return false };
    re.ty.is_integer(sema) && try_fold(sema, ctx, node).is_some_and(|v| v.is_zero())
}

fn is_void_pointer(sema: &Sema, qualif: QualifiedType) -> bool {
    let ResolvedType::Pointer(inner) = qualif.id.resolve(sema) else { return false };
    inner.is_void(sema) && !qualif.is_const && !qualif.is_volatile && !inner.is_const && !inner.is_volatile
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
    let (qty, _) = binary_op(sema, ctx, &BinaryOp::Add, e1, e2)?;
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

fn inc_dec(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> R {
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    check_assignable(re)?;
    if let ResolvedType::Pointer(inner) = re.ty.id.resolve(sema)
        && !inner.is_object(sema)
    {
        return Err(match inner.is_function(sema) {
            true => Diagnosis::BadPostIncDec(op, re.ty),
            false => Diagnosis::IncompleteType(*inner),
        });
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
    let id = sema.binding(e.id);
    let mut ops = Operands::take(sema, [e])?;
    let (sema, [re]) = ops.parts();
    address_type(sema, re, id)
}

fn address_type(sema: &mut Sema, re: &mut ResolvedExpression, sym: Option<SymbolId>) -> R {
    if re.kind == RValue && !re.casted_ty().is_function(sema) {
        return Err(Diagnosis::RValueAddress(re.ty));
    }
    if let Some(id) = sym {
        let sym = id.resolve(sema);
        if sym.storage == Some(Storage::Register) {
            return Err(Diagnosis::RegisterAddress);
        }
    }
    if is_bit_field(sema, sym) {
        return Err(Diagnosis::BitFieldAddress);
    }
    let ty = sema.types.pointer(re.ty);
    let qty = QualifiedType::new(ty, false, false);
    Ok((qty, RValue))
}

fn is_bit_field(sema: &Sema, sym: Option<SymbolId>) -> bool {
    let Some(id) = sym else { return false };
    let sym = id.resolve(sema);
    sym.kind == SymbolKind::Member && sym.value.is_some()
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
    let kind = if inner.is_function(sema) { RValue } else { LValue };
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

fn size_of_e(sema: &mut Sema, node: &ExpressionNode, e: &ExpressionNode) -> R {
    let ty = {
        let mut ops = Operands::take(&mut *sema, [e])?;
        let (sema, [re]) = ops.parts();
        if is_bit_field(sema, sema.binding(e.id)) {
            return Err(Diagnosis::SizeofBitfield);
        }
        re.ty
    };
    let result = size_t(sema, ty)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

fn size_of_ty(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, ty: &Type, span: &Span) -> R {
    let base = declaration::base_type(sema, ctx, &ty.specifiers, span);
    let (ty, _) = declaration::declared_type(sema, ctx, base, &ty.declarator).ok_poisoned()?;
    let result = size_t(sema, ty)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
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

/// 6.3.3.4 The sizeof operator yields the size in bytes of its operand, as a constant of type
/// size_t: recording it here lets 6.4 accept it as a constant expression without re-deriving the
/// type, and without evaluating the operand.
fn set_sizeof_constant(sema: &mut Sema, node: &ExpressionNode, ty: QualifiedType) {
    if let Some(layout) = layout::of(sema, ty.id)
        && let Some(value) = sema
            .target
            .cast(&sema.target.size_t, Value::UnsignedLong(layout.size.into()))
    {
        sema.set_constant(node.id, Some(value));
    }
}

fn binary_op(sema: &mut Sema, ctx: &Context, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
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
        ) => coumpound_assignment(sema, ctx, op, e1, e2),
        _ => unreachable!(),
    }
}

fn multiplicative(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    multiplicative_types(sema, op, lhs, rhs)
}

fn multiplicative_types(
    sema: &mut Sema,
    op: &BinaryOp,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
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

fn additive(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    additive_types(sema, op, lhs, rhs)
}

fn additive_types(sema: &mut Sema, op: &BinaryOp, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
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

fn shift(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let count = try_fold(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    shift_types(sema, lhs, rhs, count)
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

fn relational(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    relational_type(sema, lhs, rhs)
}

fn relational_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
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
        return Err(Diagnosis::InvalidComparison(lhs.ty, rhs.ty));
    }
    if i1.is_object(sema) && i2.is_object(sema) {
        return ret;
    }
    if !i1.is_complete(sema) && !i2.is_complete(sema) {
        return ret;
    }
    if i1.is_function(sema) && i2.is_function(sema) {
        return Err(Diagnosis::OrderedFunctionPointers(lhs.ty, rhs.ty));
    }
    Err(Diagnosis::MixedCompletenessComparison(lhs.ty, rhs.ty))
}

fn equality(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let null1 = is_null_pointer_constant(sema, ctx, e1);
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    equality_type(sema, lhs, rhs, null1, null2)
}

fn equality_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression, n1: bool, n2: bool) -> R {
    let l_ty = lhs.casted_ty();
    let r_ty = rhs.casted_ty();
    let l = l_ty.id.resolve(sema);
    let r = r_ty.id.resolve(sema);
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    let ret = Ok((ty, RValue));
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::usual_arithmetic(sema, lhs, rhs)?;
        return ret;
    }
    if l.is_pointer() && n2 {
        cast::convert(sema, rhs, l_ty.id, true);
        return ret;
    }
    if r.is_pointer() && n1 {
        cast::convert(sema, lhs, r_ty.id, true);
        return ret;
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    };
    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return ret;
    }
    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, l_ty.id, false);
        return ret;
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, r_ty.id, false);
        return ret;
    }
    Err(Diagnosis::InvalidComparison(lhs.ty, rhs.ty))
}

fn bitwise(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    bitwise_type(sema, lhs, rhs)
}

fn bitwise_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
    if !lhs.casted_ty().is_integral(sema) || !rhs.casted_ty().is_integral(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    cast::usual_arithmetic(sema, lhs, rhs)
}

fn logic(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, lhs, &e1.span);
    cast::lvalue_conversion(sema, rhs, &e2.span);
    logic_type(sema, lhs, rhs)
}

fn logic_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
    if !lhs.casted_ty().is_scalar(sema) || !rhs.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    Ok((ty, RValue))
}

fn conditional(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode, e3: &ExpressionNode) -> R {
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    let null3 = is_null_pointer_constant(sema, ctx, e3);
    let mut ops = Operands::take(sema, [e1, e2, e3])?;
    let (sema, [condition, lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, condition, &e1.span);
    cast::lvalue_conversion(sema, lhs, &e2.span);
    cast::lvalue_conversion(sema, rhs, &e3.span);
    conditional_type(sema, condition, lhs, rhs, null2, null3)
}

fn conditional_type(
    sema: &mut Sema,
    conditional: &mut ResolvedExpression,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    n2: bool,
    n3: bool,
) -> R {
    if !conditional.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::NotScalar(conditional.ty));
    }
    let l_ty = lhs.casted_ty();
    let r_ty = rhs.casted_ty();
    let l = l_ty.id.resolve(sema);
    let r = r_ty.id.resolve(sema);
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        return cast::usual_arithmetic(sema, lhs, rhs);
    }
    if l.is_tag() && r.is_tag() && l_ty.is_compatible(sema, &r_ty) {
        return Ok((l_ty, RValue));
    }
    if l.is_void() && r.is_void() {
        let ty = QualifiedType::new(sema.builtins.void, false, false);
        return Ok((ty, RValue));
    }

    if l.is_pointer() && n3 {
        cast::convert(sema, rhs, l_ty.id, true);
        return Ok((lhs.casted_ty(), RValue));
    }
    if r.is_pointer() && n2 {
        cast::convert(sema, lhs, r_ty.id, true);
        return Ok((lhs.casted_ty(), RValue));
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::IncompatibleOperands(lhs.ty, rhs.ty));
    };

    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        let (i1, i2) = (*i1, *i2);
        let inner = i1.unqualified().composite(sema, &i2.unqualified()).ok_poisoned()?;
        let inner = QualifiedType::new(inner.id, i1.is_const || i2.is_const, i1.is_volatile || i2.is_volatile);
        let ty = sema.types.pointer(inner);
        return Ok((QualifiedType::new(ty, false, false), RValue));
    }

    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, l_ty.id, false);
        return Ok((lhs.casted_ty(), RValue));
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, r_ty.id, false);
        return Ok((lhs.casted_ty(), RValue));
    }
    Err(Diagnosis::PointerMismatch(lhs.ty, rhs.ty))
}

fn cast(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, ty_node: &Type, operand: &ExpressionNode) -> R {
    let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(sema, ctx, operand);
    let mut ops = Operands::take(sema, [operand])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &operand.span);
    cast_type(sema, qualif, re, is_null)
}

fn cast_type(sema: &mut Sema, qualif: QualifiedType, re: &mut ResolvedExpression, is_null: bool) -> R {
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

fn simple_assignment(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let is_null = is_null_pointer_constant(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    simple_assignation_type(sema, lhs, rhs, is_null)
}

fn simple_assignation_type(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    is_null: bool,
) -> R {
    check_assignable(lhs)?;
    let q = cast::assignment_conversion(sema, lhs, rhs, is_null, AssignmentContext::Assignment)?;
    Ok((q, ExpressionKind::RValue))
}

fn additive_assignment(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    additive_assignation_type(sema, lhs, rhs, &e1.span)
}

fn additive_assignation_type(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    span: &Span,
) -> R {
    check_assignable(lhs)?;
    let target = lhs.ty.unqualified();
    let l = lhs.ty.id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);

    if matches!(l, ResolvedType::Pointer(inner) if inner.is_object(sema)) && r.is_integral(sema) {
        cast::lvalue_conversion(sema, lhs, span);
        cast::pointer_integer_arithmetic(sema, lhs, rhs)?;
    } else if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::lvalue_conversion(sema, lhs, span);
        cast::usual_arithmetic(sema, lhs, rhs)?;
        lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
    } else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.casted_ty()));
    }
    Ok((target, ExpressionKind::RValue))
}

fn coumpound_assignment(sema: &mut Sema, ctx: &Context, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let count = try_fold(sema, ctx, e2);
    let mut ops = Operands::take(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    coumpound_assignation_type(sema, op, lhs, rhs, &e1.span, count)
}

fn coumpound_assignation_type(
    sema: &mut Sema,
    op: &BinaryOp,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    span: &Span,
    count: Option<Value>,
) -> R {
    check_assignable(lhs)?;
    let target = lhs.ty.unqualified();
    cast::lvalue_conversion(sema, lhs, span);
    match op {
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative_types(sema, op, lhs, rhs),
        BinaryOp::Left | BinaryOp::Right => shift_types(sema, lhs, rhs, count),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise_type(sema, lhs, rhs),
        _ => unreachable!(),
    }?;
    lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
    Ok((target, ExpressionKind::RValue))
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
        Expression::SizeofExpr(e) => size_of_e(sema, node, e),
        Expression::SizeofType(ty) => size_of_ty(sema, ctx, node, ty, &node.span),
        Expression::Binary(op, e1, e2) => binary_op(sema, ctx, op, e1, e2),
        Expression::Ternary(e1, e2, e3) => conditional(sema, ctx, e1, e2, e3),
        Expression::Assign(op, e1, e2) => assignment(sema, ctx, op, e1, e2),
        Expression::Cast(ty_node, operand) => cast(sema, ctx, node, ty_node, operand),
        Expression::List(es) => list(sema, es),
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
