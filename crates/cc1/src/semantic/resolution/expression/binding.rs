use crate::ast::{Expression, ExpressionNode};
use crate::context::Context;
use crate::semantic::resolution::declaration;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diag, DiagCollector, Diagnosis, SymbolResolver};

pub fn bind_callee(resolver: &mut SymbolResolver, ctx: &Context, node: &ExpressionNode) {
    if let Expression::FunctionCall(f, _) = node.id.resolve()
        && let Expression::Identifier(fn_name) = f.id.resolve()
        && resolver.lookup_ordinary(fn_name.id).is_none()
    {
        declaration::implicit_declare_function(resolver, fn_name, &node.span);
    }
}

pub fn bind(resolver: &mut SymbolResolver, ctx: &Context, node: &ExpressionNode) {
    if resolver.sema.expr_bindings.seen(node.id) {
        return;
    }
    if let Expression::Identifier(name) = node.id.resolve() {
        let sym = resolver.lookup_ordinary(name.id);
        if sym.is_none() {
            resolver.add_diag(Diag::err((), Diagnosis::UndeclaredIdentifier(*name)), &node.span);
        }
        resolver.sema.expr_bindings.set(node.id, sym);
    }
    resolve_expression(resolver, ctx, node);
}
