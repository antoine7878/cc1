use crate::ast::ExpressionNode;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diagnostic, ResolvedType, Sema, cast};

pub fn relational(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        let l = lhs.casted_ty().id.resolve_with(sema);
        let r = rhs.casted_ty().id.resolve_with(sema);
        let ret = int_rvalue(sema);
        if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
            cast::usual_arithmetic(sema, lhs, rhs)?;
            return ret;
        }
        let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
            return Err(Diagnostic::InvalidBinaryOperand(lhs.ty, rhs.ty));
        };
        if !i1.is_compatible_ignoring_qualifiers(sema, i2) {
            return Err(Diagnostic::InvalidComparison(lhs.ty, rhs.ty));
        }
        if i1.is_object(sema) && i2.is_object(sema) {
            return ret;
        }
        if !i1.is_complete(sema) && !i2.is_complete(sema) {
            return ret;
        }
        if i1.is_function(sema) && i2.is_function(sema) {
            return Err(Diagnostic::OrderedFunctionPointers(lhs.ty, rhs.ty));
        }
        Err(Diagnostic::MixedCompletenessComparison(lhs.ty, rhs.ty))
    })
}

pub fn equality(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    let null1 = is_null_pointer_constant(sema, e1);
    let null2 = is_null_pointer_constant(sema, e2);
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        let ret = int_rvalue(sema);
        if lhs.casted_ty().is_arithmetic(sema) && rhs.casted_ty().is_arithmetic(sema) {
            cast::usual_arithmetic(sema, lhs, rhs)?;
            return ret;
        }
        let were_pointers = both_pointers(sema, lhs, rhs);
        match reconcile_pointers(sema, lhs, rhs, null1, null2) {
            Some(_) => ret,
            None if were_pointers => Err(Diagnostic::InvalidComparison(lhs.ty, rhs.ty)),
            None => Err(Diagnostic::InvalidBinaryOperand(lhs.ty, rhs.ty)),
        }
    })
}
