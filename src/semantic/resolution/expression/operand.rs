use std::iter::zip;

use crate::arena::{Loan, OptionPoisoned, ResolveWith};
use crate::ast::{Expression, ExpressionId, ExpressionNode};
use crate::context::Context;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{
    Diagnosis, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, Sema, SymbolId, SymbolKind, cast,
    declaration, ice,
};

pub type R = Result<(QualifiedType, ExpressionKind), Diagnosis>;

type Operands<'s, const N: usize> = Loan<'s, Sema, ExpressionId, ResolvedExpression, N>;

pub fn operands<'s, const N: usize>(
    sema: &'s mut Sema,
    nodes: [&ExpressionNode; N],
) -> Result<Operands<'s, N>, Diagnosis> {
    Loan::take(sema, nodes.map(|n| n.id)).ok_poisoned()
}

pub fn int_rvalue(sema: &Sema) -> R {
    Ok((QualifiedType::plain(sema.builtins.int), RValue))
}

pub fn with_ops<const N: usize, T, F>(sema: &mut Sema, nodes: [&ExpressionNode; N], f: F) -> Result<T, Diagnosis>
where
    F: FnOnce(&mut Sema, &mut [ResolvedExpression; N]) -> Result<T, Diagnosis>,
{
    let mut ops = operands(sema, nodes)?;
    let (sema, res) = ops.parts();
    f(sema, res)
}

pub fn with_converted<const N: usize, T, F>(sema: &mut Sema, nodes: [&ExpressionNode; N], f: F) -> Result<T, Diagnosis>
where
    F: FnOnce(&mut Sema, &mut [ResolvedExpression; N]) -> Result<T, Diagnosis>,
{
    with_ops(sema, nodes, |sema, res| {
        for (re, node) in zip(res.iter_mut(), nodes) {
            cast::lvalue_conversion(sema, re, &node.span);
        }
        f(sema, res)
    })
}

pub fn is_null_pointer_constant(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> bool {
    let mut node = node;
    if let Expression::Cast(ty_node, op) = node.id.resolve(ctx) {
        let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
        if let Some((qualif, _)) = declaration::declared_type(sema, ctx, base, &ty_node.declarator)
            && is_void_pointer(sema, qualif)
        {
            node = op;
        }
    }
    let Some(re) = sema.expr_types.get(node.id) else { return false };
    re.ty.is_integer(sema) && ice::try_fold(sema, ctx, node).is_some_and(|v| v.is_zero())
}

fn is_void_pointer(sema: &Sema, qualif: QualifiedType) -> bool {
    let ResolvedType::Pointer(inner) = qualif.id.resolve(sema) else { return false };
    inner.is_void(sema) && !qualif.is_const && !qualif.is_volatile && !inner.is_const && !inner.is_volatile
}

pub fn is_bit_field(sema: &Sema, sym: Option<SymbolId>) -> bool {
    let Some(id) = sym else { return false };
    let sym = id.resolve(sema);
    sym.kind == SymbolKind::Member && sym.value.is_some()
}

pub fn check_assignable(lhs: &ResolvedExpression) -> Result<(), Diagnosis> {
    if lhs.kind == ExpressionKind::RValue {
        return Err(Diagnosis::AssignToRValue);
    }
    if lhs.ty.is_const {
        return Err(Diagnosis::ConstAssignment(lhs.ty));
    }
    Ok(())
}
