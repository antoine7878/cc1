use std::mem::take;

use crate::arena::{ResolveMutWith, ResolveWith};
use crate::ast::statement::StatementId;
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_jump_statement, walk_labeled_statement,
    walk_statement, walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, Expression, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode,
    InitializerNode, IterationStatement, IterationStatementNode, JumpStatementNode, LabeledStatementNode, Name,
    SelectionStatement, SelectionStatementNode, Statement, StatementNode, StringId, Tag, Value,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::model::initializer;
use crate::semantic::resolution::{expression, statement};
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefId, Linkage, QualifiedType, ResolvedStatement,
    ResolvedType, ScopeKind, Sema, StatementScopes, Symbol, SymbolId, SymbolKind, SymbolScopes, TagDefId, constrain,
    declaration, ice,
};

#[derive(Debug)]
pub struct SymbolResolver<'a> {
    pub sema: &'a mut Sema,
    sym_scopes: SymbolScopes,
    pub stmt_scopes: StatementScopes,
    f: Option<CurrentFunction>,
    gotos: Vec<Name>,
}

#[derive(Clone, Copy, Debug)]
struct CurrentFunction {
    def: FunctionDefId,
    return_ty: QualifiedType,
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
            sym_scopes: SymbolScopes::default(),
            stmt_scopes: StatementScopes::default(),
            f: None,
            gotos: Vec::new(),
        }
    }

    pub fn resolve_unit(sema: &mut Sema, ctx: &Context) {
        let mut resolver = SymbolResolver::new(sema);
        resolver.sym_scopes.push(ScopeKind::File);
        walk_translation_unit(&mut resolver, ctx, &ctx.ast);
        resolver.sym_scopes.pop();
        debug_assert!(resolver.sym_scopes.is_empty());
    }
}

// ----- Scopes ----------------------------

impl SymbolResolver<'_> {
    pub fn scope_kind(&self) -> ScopeKind {
        self.sym_scopes.kind()
    }

    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.sym_scopes.push(kind);
    }

    pub fn leave_scope(&mut self) {
        self.sym_scopes.pop();
    }

    pub fn promote_scope(&mut self, kind: ScopeKind) {
        self.sym_scopes.set_kind(kind);
    }

    pub fn lookup_ordinary(&self, name: StringId) -> Option<SymbolId> {
        self.sym_scopes.lookup_ordinary(name)
    }

    pub fn current(&self, name: StringId) -> Option<SymbolId> {
        self.sym_scopes.current(name)
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
        let sym_id = self.sym_scopes.lookup_ordinary(name.id)?;
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

        if let Some(id) = self.sym_scopes.lookup_tag(name.id, is_definition) {
            let def = id.resolve(self.sema);
            if def.kind != kind || (is_definition && def.is_complete) {
                self.add_diag(Diag::err((), Diagnosis::DuplicateDeclaration(def.kind(), name)), span)
            }
            return id;
        }

        let id = self.sema.tags.declare(kind, Some(name));
        self.sym_scopes.insert_tag(name.id, id);
        id
    }

    pub fn declare(&mut self, sym: Symbol, span: &Span) -> SymbolId {
        let name = sym.name;
        let lexical = self.dedup(&sym, span);
        let sym_id = if sym.linkage != Linkage::None {
            self.sema.register_external(sym, span, lexical)
        } else {
            lexical.unwrap_or_else(|| self.sema.symbols.alloc(sym))
        };
        self.sym_scopes.insert(name.id, sym_id);
        sym_id
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = self.sym_scopes.current(sym.name.id)?;
        let old_symbol = old_id.resolve(self.sema);
        if sym.kind != SymbolKind::Typedef
            && (self.sym_scopes.kind() == ScopeKind::File
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
}

// ----- Labels ----------------------------

impl SymbolResolver<'_> {
    pub fn define_label(&mut self, name: Name, span: &Span) {
        let f = self.f.expect("a label inside a function");
        if self.labels(f).any(|label| label == name.id) {
            return self.add_diag(Diag::err((), Diagnosis::DuplicateLabel(name)), span);
        }
        self.sema.functions.get_mut(f.def).labels.push(name);
    }

    pub fn reference_label(&mut self, name: Name) {
        self.gotos.push(name);
    }

    fn labels(&self, f: CurrentFunction) -> impl Iterator<Item = StringId> {
        self.sema.functions.get(f.def).labels.iter().map(|label| label.id)
    }

    fn resolve_gotos(&mut self, f: CurrentFunction) {
        let defined: Vec<StringId> = self.labels(f).collect();
        for goto in take(&mut self.gotos) {
            if !defined.contains(&goto.id) {
                self.add_diag(Diag::err((), Diagnosis::UndefinedLabel(goto)), &goto.span);
            }
        }
    }
}

// ----- Resolution ------------------------

impl SymbolResolver<'_> {
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
            let sym = self.sym_scopes.lookup_ordinary(name.id);
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

    fn iteration_statement(&mut self, ctx: &Context, id: StatementId, inner: &IterationStatementNode) {
        let body = self.loop_controls(ctx, inner);
        statement::check_iteration_statement(self.sema, inner);
        self.stmt_scopes.push_loop(id);
        self.visit_statement(ctx, body);
        self.leave_stmt_scope();
    }

    fn loop_controls<'n>(&mut self, ctx: &Context, inner: &'n IterationStatementNode) -> &'n StatementNode {
        match &inner.stmt {
            IterationStatement::While(e, body) | IterationStatement::Do(body, e) => {
                self.visit_expression(ctx, e);
                body
            }
            IterationStatement::For(b) => {
                let (init, condition, step, body) = &**b;
                for e in [&init.expr, &condition.expr, step].into_iter().flatten() {
                    self.visit_expression(ctx, e);
                }
                body
            }
        }
    }

    fn leave_stmt_scope(&mut self) {
        let scope = self.stmt_scopes.pop().expect("a statement scope to leave");
        let (id, rs): (StatementId, ResolvedStatement) = scope.into();
        self.sema.stmts.set(id, Some(rs));
    }

    fn labeled_statement(&mut self, ctx: &Context, id: StatementId, inner: &LabeledStatementNode) {
        statement::check_labeled_statement(self, ctx, id, inner);
        walk_labeled_statement(self, ctx, inner);
    }

    fn jump_statement(&mut self, ctx: &Context, id: StatementId, inner: &JumpStatementNode) {
        walk_jump_statement(self, ctx, inner);
        statement::check_jump_statement(self, ctx, id, inner, self.f.map(|f| f.return_ty));
    }

    fn selection_statement(&mut self, ctx: &Context, id: StatementId, inner: &SelectionStatementNode) {
        match &inner.stmt {
            SelectionStatement::If(condition, then, otherwise) => {
                self.visit_expression(ctx, condition);
                statement::check_selection_statement(self.sema, inner);
                self.visit_statement(ctx, then);
                if let Some(otherwise) = otherwise {
                    self.visit_statement(ctx, otherwise);
                }
            }
            SelectionStatement::Switch(condition, body) => {
                self.visit_expression(ctx, condition);
                let control = statement::check_selection_statement(self.sema, inner);
                self.stmt_scopes.push_switch(id, control, vec![], None);
                self.visit_statement(ctx, body);
                self.leave_stmt_scope();
            }
        }
    }
}

impl Visitor for SymbolResolver<'_> {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(header) = declaration::define_function(self, ctx, node) else { return };
        let f = CurrentFunction {
            def: header.id,
            return_ty: header.return_ty,
        };
        self.f = Some(f);
        declaration::bind_function_parameters(self, ctx, node, &header);
        self.visit_compound_statement(ctx, &node.body);
        self.resolve_gotos(f);
        self.f = None;
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

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        statement::enter_compound_statement(self);
        walk_compound_statement(self, ctx, node);
        statement::leave_compound_statement(self);
    }

    fn visit_statement(&mut self, ctx: &Context, node: &crate::ast::StatementNode) {
        match node.id.resolve(ctx) {
            Statement::Iteration(inner) => self.iteration_statement(ctx, node.id, inner),
            Statement::Selection(inner) => self.selection_statement(ctx, node.id, inner),
            Statement::Labeled(inner) => self.labeled_statement(ctx, node.id, inner),
            Statement::Jump(inner) => self.jump_statement(ctx, node.id, inner),
            _ => walk_statement(self, ctx, node),
        }
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if let Expression::FunctionCall(f, _) = node.id.resolve(ctx)
            && let Expression::Identifier(fn_name) = f.id.resolve(ctx)
            && self.sym_scopes.lookup_ordinary(fn_name.id).is_none()
        {
            declaration::implicit_declare_function(self, fn_name, &node.span);
        }
        walk_expression(self, ctx, node);
        self.resolve_expression(ctx, node);
    }
}
