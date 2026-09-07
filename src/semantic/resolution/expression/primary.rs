use std::iter::zip;

use crate::arena::{OptionPoisoned, ResolveWith};
use crate::ast::{BinaryOp, ExpressionNode, MemberOp, Name, Tag};
use crate::context::Context;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::resolution::expression::*;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, MemberRef, ParamTypes, QualifiedType, ResolvedType, Sema, cast,
};

pub fn identifier(sema: &mut Sema, node: &ExpressionNode) -> R {
    let id = sema.expr_bindings.get(node.id).copied().ok_poisoned()?;
    let sym = id.resolve(sema);
    Ok((sym.ty.ok_poisoned()?, sym.expression_kind()))
}

pub fn constant(sema: &mut Sema, e: &ExpressionNode) -> R {
    sema.expr_types
        .get(e.id)
        .map(|re| (re.casted_ty(), re.kind))
        .ok_poisoned()
}

pub fn array_subscript(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let (qty, _) = binary_op(sema, ctx, &BinaryOp::Add, e1, e2)?;
    let ResolvedType::Pointer(inner) = qty.id.resolve(sema) else {
        return Err(Diagnosis::SubscriptNotArray);
    };
    Ok((*inner, LValue))
}

pub fn fn_call(sema: &mut Sema, ctx: &Context, fn_node: &ExpressionNode, args: &[ExpressionNode]) -> R {
    with_converted(sema, [fn_node], |sema, [re]| {
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
            let promoted = with_converted(&mut *sema, [arg_node], |sema, [re]| {
                cast::default_argument_promotions(sema, re);
                Ok(())
            });
            if let Err(err) = promoted {
                sema.add_diag(Diag::err((), err), &arg_node.span);
                rejected = true;
            }
        }
        match rejected {
            true => Err(Diagnosis::Poisoned),
            false => Ok((QualifiedType::plain(ret.id), RValue)),
        }
    })
}

pub fn member(sema: &mut Sema, node: &ExpressionNode, op: MemberOp, e: &ExpressionNode, name: &Name) -> R {
    with_ops(sema, [e], |sema, [re]| {
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
        let Some((index, sym_id)) = tag.find_member(sema, name) else {
            return Err(Diagnosis::AccessNotMember(tag_qty, name.id));
        };
        sema.expr_bindings.set(node.id, Some(sym_id));
        sema.member_refs.set(node.id, Some(MemberRef { tag: tag_id, index }));
        let sym = sym_id.resolve(sema);
        let ty = sym.ty.ok_or(Diagnosis::Poisoned)?;
        let qty = QualifiedType::new(
            ty.id,
            ty.is_const || tag_qty.is_const,
            ty.is_volatile || tag_qty.is_volatile,
        );
        Ok((qty, kind))
    })
}
