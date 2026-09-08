use std::mem::take;

use crate::arena::ResolveWith;
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_statement, walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, Name,
    Statement, StatementNode, StringId, Tag, Value,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::{expression, statement};
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefId, Linkage, QualifiedType, ScopeKind, Sema,
    StatementScopes, Symbol, SymbolId, SymbolKind, SymbolScopes, TagDefId, constrain, declaration, ice,
};

#[derive(Debug)]
pub struct SymbolResolver<'a> {
    pub sema: &'a mut Sema,
    sym_scopes: SymbolScopes,
    pub stmt_scopes: StatementScopes,
    f: Option<FunctionDefId>,
    gotos: Vec<Name>,
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
        self.sema.functions.get_mut(f).labels.push(name);
    }

    pub fn return_ty(&self) -> Option<QualifiedType> {
        self.f.map(|f| self.sema.functions.get(f).return_ty)
    }

    pub fn reference_label(&mut self, name: Name) {
        self.gotos.push(name);
    }

    fn labels(&self, f: FunctionDefId) -> impl Iterator<Item = StringId> {
        self.sema.functions.get(f).labels.iter().map(|label| label.id)
    }

    fn resolve_gotos(&mut self, f: FunctionDefId) {
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
}

impl Visitor for SymbolResolver<'_> {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(header) = declaration::define_function(self, ctx, node) else { return };
        let f = header.id;
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
            declaration::resolve_initializer(self, ctx, sym, ty, init);
        }
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        statement::enter_compound_statement(self);
        walk_compound_statement(self, ctx, node);
        statement::leave_compound_statement(self);
    }

    fn visit_statement(&mut self, ctx: &Context, node: &StatementNode) {
        match node.id.resolve(ctx) {
            Statement::Iteration(inner) => statement::resolve_iteration_statement(self, ctx, node.id, inner),
            Statement::Selection(inner) => statement::resolve_selection_statement(self, ctx, node.id, inner),
            Statement::Labeled(inner) => statement::resolve_labeled_statement(self, ctx, node.id, inner),
            Statement::Jump(inner) => statement::resolve_jump_statement(self, ctx, node.id, inner),
            _ => walk_statement(self, ctx, node),
        }
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        expression::bind_callee(self, ctx, node);
        walk_expression(self, ctx, node);
        expression::bind(self, ctx, node);
    }
}
