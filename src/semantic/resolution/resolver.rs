use crate::arena::{ResolveMutWith, ResolveWith};
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_iteration_statement, walk_jump_statement,
    walk_labeled_statement, walk_selection_statement, walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, Expression, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode,
    InitializerNode, IterationStatementNode, JumpStatementNode, LabeledStatementNode, Name, SelectionStatementNode,
    StringId, Tag, Value,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::model::initializer;
use crate::semantic::resolution::{expression, statement};
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, Linkage, QualifiedType, ResolvedType, ScopeKind, Scopes, Sema,
    Symbol, SymbolId, SymbolKind, TagDefId, constrain, declaration, ice,
};

#[derive(Debug)]
pub struct SymbolResolver<'a> {
    pub sema: &'a mut Sema,
    scopes: Scopes,
    return_ty: Option<QualifiedType>,
}

impl DiagCollector for SymbolResolver<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.sema.diagnosis
    }
}

impl<'a> SymbolResolver<'a> {
    pub fn new(sema: &'a mut Sema) -> Self {
        Self {
            sema,
            scopes: Scopes::default(),
            return_ty: None,
        }
    }

    pub fn resolve_unit(sema: &mut Sema, ctx: &Context) {
        let mut resolver = SymbolResolver::new(sema);
        resolver.scopes.push(ScopeKind::File);
        walk_translation_unit(&mut resolver, ctx, &ctx.ast);
        resolver.scopes.pop();
        debug_assert!(resolver.scopes.is_empty());
    }
}

// ----- Scopes ----------------------------

impl SymbolResolver<'_> {
    pub fn scope_kind(&self) -> ScopeKind {
        self.scopes.kind()
    }

    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.scopes.push(kind);
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn promote_scope(&mut self, kind: ScopeKind) {
        self.scopes.set_kind(kind);
    }

    pub fn lookup_ordinary(&self, name: StringId) -> Option<SymbolId> {
        self.scopes.lookup_ordinary(name)
    }

    pub fn current(&self, kind: SymbolKind, name: StringId) -> Option<SymbolId> {
        self.scopes.current(kind, name)
    }
}

// ----- Binding ---------------------------

impl SymbolResolver<'_> {
    pub fn resolve_typedef(
        &mut self,
        name: Name,
        is_const: bool,
        is_volatile: bool,
        span: &Span,
    ) -> Option<QualifiedType> {
        let sym_id = self.scopes.lookup_ordinary(name.id)?;
        let sym = sym_id.resolve(self.sema);
        if sym.kind != SymbolKind::Typedef {
            return self.add_diag(Diag::err(None, Diagnosis::UndeclaredIdentifier(name)), span);
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.add_diag(Diag::err((), Diagnosis::DuplicateTypeQualifiers), span)
        }
        Some(QualifiedType::new(
            base.id,
            base.is_const || is_const,
            base.is_volatile || is_volatile,
        ))
    }

    pub fn declare_tag(&mut self, kind: Tag, name: Option<Name>, is_definition: bool, span: &Span) -> TagDefId {
        let Some(name) = name else { return self.sema.tags.declare(kind, None) };

        if let Some(id) = self.scopes.lookup_tag(name.id, is_definition) {
            let def = id.resolve(self.sema);
            if def.kind != kind || (is_definition && def.is_complete) {
                self.add_diag(Diag::err((), Diagnosis::DuplicateDeclaration(def.kind(), name)), span)
            }
            return id;
        }

        let id = self.sema.tags.declare(kind, Some(name));
        self.scopes.insert_tag(name.id, id);
        id
    }

    pub fn declare(&mut self, sym: Symbol, span: &Span) -> SymbolId {
        let (name, kind) = (sym.name, sym.kind);
        let lexical = self.dedup(&sym, span);
        let sym_id = if sym.linkage != Linkage::None {
            self.sema.register_external(sym, span, lexical)
        } else {
            lexical.unwrap_or_else(|| self.sema.symbols.alloc(sym))
        };
        self.scopes.insert(kind, name.id, sym_id);
        sym_id
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = self.scopes.current(sym.kind, sym.name.id)?;
        let old_symbol = old_id.resolve(self.sema);
        if (self.scopes.kind() == ScopeKind::File
            || (sym.linkage != Linkage::None && old_symbol.linkage != Linkage::None))
            && old_symbol.is_compatible(self.sema, sym)
            && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.add_diag(
            Diag::err(Some(old_id), Diagnosis::DuplicateDeclaration(sym.kind, sym.name)),
            span,
        )
    }

    pub fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(old) = self.scopes.lookup_label(name.id) {
            let old_init = old.resolve(self.sema).is_init;
            if !old_init && is_init {
                old.resolve_mut(self.sema).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.add_diag(
                Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Label, name)),
                span,
            );
        };
        let sym_id = self.sema.symbols.alloc(Symbol::label(name, is_init));
        self.scopes.insert(SymbolKind::Label, name.id, sym_id);
    }
}

// ----- Resolution ------------------------

impl SymbolResolver<'_> {
    /// Resolves `expr` in the current scope, then folds it. `ice::eval_constant` only folds, so
    /// every constant evaluated while binding is still in progress goes through here.
    pub fn eval_constant(&mut self, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
        if self.sema.expr_consts.seen(expr.id) {
            return self.sema.expr_consts.get(expr.id).copied();
        }
        self.visit_expression(ctx, expr);
        ice::eval_constant(self.sema, ctx, expr)
    }

    fn resolve_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.sema.expr_bindings.seen(node.id) {
            return;
        }
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            let sym = self.scopes.lookup_ordinary(name.id);
            if sym.is_none() {
                self.add_diag(Diag::err((), Diagnosis::UndeclaredIdentifier(*name)), &node.span);
            }
            self.sema.expr_bindings.set(node.id, sym);
        }
        expression::resolve_expression(self, ctx, node);
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
        let Some(header) = declaration::define_function(self, ctx, node) else { return };
        self.return_ty = Some(header.return_ty);
        declaration::bind_function_parameters(self, ctx, node, &header);
        self.visit_compound_statement(ctx, &node.body);
        self.return_ty = None;
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        declaration::check_declaration(self, ctx, node);
        let declared_storage = constrain::specifier::get_storage(specifiers).collect(self, span);
        let qualif = declaration::base_type(self, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            declaration::declare_init_declarator(self, ctx, init_declarator, qualif, declared_storage);
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
        statement::check_labeled_statement(self, ctx, node);
        walk_labeled_statement(self, ctx, node);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        statement::enter_compound_statement(self);
        walk_compound_statement(self, ctx, node);
        statement::leave_compound_statement(self);
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
        statement::check_jump_statement(self, ctx, node, self.return_ty);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if let Expression::FunctionCall(f, _) = node.id.resolve(ctx)
            && let Expression::Identifier(fn_name) = f.id.resolve(ctx)
            && self.scopes.lookup_ordinary(fn_name.id).is_none()
        {
            declaration::implicit_declare_function(self, fn_name, &node.span);
        }
        walk_expression(self, ctx, node);
        self.resolve_expression(ctx, node);
    }
}
