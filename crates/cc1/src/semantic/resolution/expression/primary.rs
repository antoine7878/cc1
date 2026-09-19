use std::iter::zip;

use crate::arena::OptionPoisoned;
use crate::ast::{BinaryOp, ExpressionNode, MemberOp, Name, Tag};
use crate::semantic::ValueCategory::{LValue, RValue};
use crate::semantic::resolution::expression::*;
use crate::semantic::{
    AssignmentContext, Diag, Diagnostic, DiagnosticSink, MemberRef, ParamTypes, QualifiedType, ResolvedType, Sema, cast,
};

pub fn identifier(sema: &mut Sema, node: &ExpressionNode) -> ExprResult {
    let id = sema.expr_bindings.get(node.id).copied().ok_poisoned()?;
    let sym = id.resolve_with(sema);
    Ok((sym.ty, sym.value_category()))
}

pub fn constant(sema: &mut Sema, e: &ExpressionNode) -> ExprResult {
    sema.expressions.get(e.id).map(|re| (re.casted_ty(), re.kind)).ok_poisoned()
}

pub fn array_subscript(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    let (qty, _) = binary_op(sema, &BinaryOp::Add, e1, e2)?;
    let ResolvedType::Pointer(inner) = qty.id.resolve_with(sema) else {
        return Err(Diagnostic::SubscriptNotArray);
    };
    Ok((*inner, LValue))
}

pub fn function_call(sema: &mut Sema, fn_node: &ExpressionNode, args: &[ExpressionNode]) -> ExprResult {
    with_converted(sema, [fn_node], |sema, [re]| {
        let ty = re.casted_ty();
        let ResolvedType::Pointer(inner) = ty.id.resolve_with(sema) else {
            return Err(Diagnostic::CallingNotFunction(ty));
        };
        let ResolvedType::Function { ret, params } = inner.id.resolve_with(sema).clone() else {
            return Err(Diagnostic::CallingNotFunction(ty));
        };
        let returned = ret.id.resolve_with(sema);
        if !returned.is_void() && !returned.is_complete(sema) {
            return Err(Diagnostic::CallingIncompleteReturn(ret));
        }
        let (params, is_variadic) = match params {
            ParamTypes::Prototype { params, is_variadic } => (params, is_variadic),
            ParamTypes::Unspecified => (vec![], true),
        };
        if !is_variadic && args.len() > params.len() {
            return Err(Diagnostic::TooManyArguments(params.len(), args.len()));
        }
        if args.len() < params.len() {
            return Err(Diagnostic::TooFewArguments(params.len(), args.len()));
        }
        let param_len = params.len();
        let mut rejected = false;
        for (n, (param_ty, arg_node)) in zip(params, args).enumerate() {
            if let Err(err) = init(sema, param_ty, arg_node, AssignmentContext::Argument(n + 1)) {
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
            true => Err(Diagnostic::Poisoned),
            false => Ok((QualifiedType::plain(ret.id), RValue)),
        }
    })
}

pub fn member(sema: &mut Sema, node: &ExpressionNode, op: MemberOp, e: &ExpressionNode, name: &Name) -> ExprResult {
    with_ops(sema, [e], |sema, [re]| {
        let (&tag_qty, kind) = match op {
            MemberOp::Dot => (&re.casted_ty(), re.kind),
            MemberOp::Arrow => {
                cast::convert_operand(sema, re, &e.span);
                let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve_with(sema) else {
                    return Err(Diagnostic::AccessNotPointer(re.casted_ty()));
                };
                (inner, LValue)
            }
        };
        let &ResolvedType::Tag(tag_id) = tag_qty.id.resolve_with(sema) else {
            return Err(Diagnostic::AccessNotStuctOrUnion(tag_qty));
        };
        let tag = tag_id.resolve_with(sema);
        if !tag.is_complete {
            return Err(Diagnostic::IncompleteType(tag_qty));
        }
        if tag.kind != Tag::Struct && tag.kind != Tag::Union {
            return Err(Diagnostic::AccessNotStuctOrUnion(tag_qty));
        }
        let Some((index, sym_id)) = tag.find_member(sema, name) else {
            return Err(Diagnostic::AccessNotMember(tag_qty, name.id));
        };
        sema.expr_bindings.set(node.id, Some(sym_id));
        sema.member_refs.set(node.id, Some(MemberRef { tag: tag_id, index }));
        let sym = sym_id.resolve_with(sema);
        let ty = sym.ty;
        let qty = QualifiedType::new(ty.id, ty.is_const || tag_qty.is_const, ty.is_volatile || tag_qty.is_volatile);
        Ok((qty, kind))
    })
}
