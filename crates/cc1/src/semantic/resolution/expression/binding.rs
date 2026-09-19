use crate::ast::{Expression, ExpressionNode};
use crate::semantic::resolution::declaration;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diag, Diagnostic, DiagnosticSink, Resolver};

pub fn bind_callee(resolver: &mut Resolver, node: &ExpressionNode) {
    if let Expression::FunctionCall(f, _) = node.id.resolve()
        && let Expression::Identifier(fn_name) = f.id.resolve()
        && resolver.lookup_ordinary(fn_name.id).is_none()
    {
        declaration::implicit_declare_function(resolver, fn_name, &node.span);
    }
}

pub fn bind(resolver: &mut Resolver, node: &ExpressionNode) {
    if resolver.sema.expr_bindings.contains(node.id) {
        return;
    }
    if let Expression::Identifier(name) = node.id.resolve() {
        let sym = resolver.lookup_ordinary(name.id);
        if sym.is_none() {
            resolver.add_diag(Diag::err((), Diagnostic::UndeclaredIdentifier(*name)), &node.span);
        }
        resolver.sema.expr_bindings.set(node.id, sym);
    }
    resolve_expression(resolver, node);
}
