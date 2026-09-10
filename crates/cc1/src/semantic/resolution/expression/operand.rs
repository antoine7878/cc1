use std::iter::zip;

use crate::arena::{Loan, OptionPoisoned};
use crate::ast::{Expression, ExpressionId, ExpressionNode};
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{
    Diagnosis, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, Sema, SymbolId, SymbolKind, cast, ice,
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

/// Operands are resolved before the parent that inspects them, so the cast type is read back from
/// the expression table instead of resolving the type name a second time.
pub fn is_null_pointer_constant(sema: &mut Sema, node: &ExpressionNode) -> bool {
    let mut node = node;
    if let Expression::Cast(_, op) = node.id.resolve()
        && let Some(re) = sema.expr_types.get(node.id)
        && is_void_pointer(sema, re.ty)
    {
        node = op;
    }
    let Some(re) = sema.expr_types.get(node.id) else { return false };
    re.ty.is_integer(sema) && ice::try_fold(sema, node).is_some_and(|v| v.is_zero())
}

fn is_void_pointer(sema: &Sema, qualif: QualifiedType) -> bool {
    let ResolvedType::Pointer(inner) = qualif.id.resolve_in(sema) else { return false };
    inner.is_void(sema) && !qualif.is_const && !qualif.is_volatile && !inner.is_const && !inner.is_volatile
}

pub fn is_bit_field(sema: &Sema, sym: Option<SymbolId>) -> bool {
    let Some(id) = sym else { return false };
    let sym = id.resolve_in(sema);
    sym.kind == SymbolKind::Member && sym.value.is_some()
}
