use crate::arena::{ResolveMutWith, ResolveWith};
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_iteration_statement, walk_jump_statement,
    walk_labeled_statement, walk_selection_statement,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, Expression, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode,
    InitializerNode, IterationStatementNode, JumpStatementNode, LabeledStatementNode, SelectionStatementNode,
};
use crate::context::Context;
use crate::semantic::model::initializer;
use crate::semantic::resolution::{expression, statement};
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, QualifiedType, ResolvedType, Sema, SymbolId, constrain, declaration,
};

#[derive(Debug)]
pub struct SymbolResolver<'a> {
    pub sema: &'a mut Sema,
    return_ty: Option<QualifiedType>,
}

impl DiagCollector for SymbolResolver<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.sema.diagnosis
    }
}

impl<'a> SymbolResolver<'a> {
    pub fn new(sema: &'a mut Sema) -> Self {
        Self { sema, return_ty: None }
    }
}

impl SymbolResolver<'_> {
    fn resolve_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.sema.expr_bindings.seen(node.id) {
            return;
        }
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            let sym = self.sema.scopes.lookup_ordinary(name.id);
            if sym.is_none() {
                self.add_diag(Diag::err((), Diagnosis::UndeclaredIdentifier(*name)), &node.span);
            }
            self.sema.expr_bindings.set(node.id, sym);
        }
        expression::resolve_expression(self.sema, ctx, node);
    }

    fn resolve_initializer(&mut self, ctx: &Context, sym_id: SymbolId, ty: QualifiedType, node: &InitializerNode) {
        let duration = sym_id.resolve(self.sema).duration;
        let init = initializer::resolve(self, ctx, ty, node, duration);
        if let ResolvedType::Array { elem, len: None } = ty.id.resolve(self.sema)
            && let Some(len) = init.len(ctx)
        {
            let id = self.sema.types.array(*elem, Some(len));
            sym_id.resolve_mut(self.sema).ty = Some(QualifiedType::new(id, ty.is_const, ty.is_volatile));
        }
        let id = self.sema.inits.alloc(init);
        sym_id.resolve_mut(self.sema).initializer = Some(id);
    }
}

impl Visitor for SymbolResolver<'_> {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(header) = declaration::define_function(self.sema, ctx, node) else { return };
        self.return_ty = Some(header.return_ty);
        declaration::bind_function_parameters(self.sema, ctx, node, &header);
        self.visit_compound_statement(ctx, &node.body);
        self.return_ty = None;
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        declaration::check_declaration(self.sema, ctx, node);
        let declared_storage = constrain::specifier::get_storage(specifiers).collect(self, span);
        let qualif = declaration::base_type(self.sema, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            declaration::declare_init_declarator(self.sema, ctx, init_declarator, qualif, declared_storage);
        }
        walk_declaration(self, ctx, node);
    }

    fn visit_init_declarator(&mut self, ctx: &Context, node: &InitDeclaratorNode) {
        let decl = &node.declarator;
        let Some(&sym) = self.sema.declarations.get(&decl.id) else { return };
        let Some(ty) = sym.resolve(self.sema).ty else { return };
        self.visit_declarator(ctx, &node.declarator);
        if let Some(init) = &node.initializer {
            self.resolve_initializer(ctx, sym, ty, init);
        }
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        statement::check_labeled_statement(self.sema, ctx, node);
        walk_labeled_statement(self, ctx, node);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        statement::enter_compound_statement(self.sema);
        walk_compound_statement(self, ctx, node);
        statement::leave_compound_statement(self.sema);
    }

    fn visit_selection_statement(&mut self, ctx: &Context, node: &SelectionStatementNode) {
        walk_selection_statement(self, ctx, node);
        statement::check_selection_statement(self.sema, node);
    }

    fn visit_iteration_statement(&mut self, ctx: &Context, node: &IterationStatementNode) {
        walk_iteration_statement(self, ctx, node);
        statement::check_iteration_statement(self.sema, node);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        walk_jump_statement(self, ctx, node);
        statement::check_jump_statement(self.sema, ctx, node, self.return_ty);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if let Expression::FunctionCall(f, _) = node.id.resolve(ctx)
            && let Expression::Identifier(fn_name) = f.id.resolve(ctx)
            && self.sema.scopes.lookup_ordinary(fn_name.id).is_none()
        {
            declaration::implicit_declare_function(self.sema, fn_name, &node.span);
        }
        walk_expression(self, ctx, node);
        self.resolve_expression(ctx, node);
    }
}
