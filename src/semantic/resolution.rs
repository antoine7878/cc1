use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_jump_statement, walk_labeled_statement,
    walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, ExpressionNode,
    FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, JumpStatement, JumpStatementNode, Labeled,
    LabeledStatementNode, Name, ParameterDeclaration, Storage, TypeSpecifier,
};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::sema::Sema;
use crate::semantic::symbol::Symbol;
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ScopeKind, SymbolId, SymbolKind, constrain,
};
use crate::target::Target;

#[derive(Default, Debug)]
pub struct SymbolResolver {
    pub sema: Sema,
}

impl DiagCollector for SymbolResolver {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.sema.diagnosis
    }
}

impl SymbolResolver {
    pub fn new(target: Target) -> Self {
        Self { sema: Sema::new(target) }
    }

    fn add_function<'a>(
        &mut self,
        ctx: &'a Context,
        node: &FunctionDefinitionNode,
    ) -> Option<&'a FunctionParametersNode> {
        let span = &node.span;

        let qualif = constrain::declaration::resolve_type(&mut self.sema, ctx, &node.specifiers, span);
        let (ty, fn_decl) = self.sema.make_qualified_type(ctx, qualif, &node.declarator)?;
        let (decl, parameters) =
            constrain::external::extract_function_declarator(fn_decl.id.resolve(ctx)).collect(self, span)?;

        let storage = constrain::declaration::get_storage(&node.specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let name = decl.ident(ctx)?;
        let sym = Symbol::new(name, Some(ty), Some(storage), SymbolKind::Function, true);
        self.sema.declare(sym, &node.declarator.span);
        Some(parameters)
    }

    fn param_empty(&mut self, lst: &[DeclarationNode], span: &Span) {
        if !lst.is_empty() {
            self.add_diag(Diag::only_diag(Diagnosis::ParameterTypeListWithList), span)
        }
    }

    #[rustfmt::skip]
    fn param_ident_style(
        &mut self,
        ctx: &Context,
        params: &[ParameterDeclaration],
        lst: &[DeclarationNode],
        span: &Span,
    ) {
        let a = constrain::external::is_valid_parameter_style(ctx, params, lst);
        match a {
            Diag { res: true, diagnosis: None, } => (),
            Diag { res: false, diagnosis: None, } => return,
            _ => { let _ = a.collect(self, span); }
        }

        for param in params {
            let ParameterDeclaration {
                span,
                specifiers,
                declarator,
            } = param;
            self.add_parameter_declarator(ctx, specifiers, declarator, span);
        }
    }

    fn param_old_style(&mut self, ctx: &Context, names: &[Name], lst: &[DeclarationNode], span: &Span) {
        let declarations: Vec<_> = lst
            .iter()
            .flat_map(
                |DeclarationNode {
                     span,
                     specifiers,
                     init_declarators,
                 }| {
                    init_declarators
                        .iter()
                        .map(|decl| {
                            self.add_parameter_declarator(ctx, specifiers, &decl.declarator, span)
                                .map(|sym_id| self.sema.symbols.get(sym_id).name.id)
                        })
                        .collect::<Vec<_>>()
                },
            )
            .collect();

        let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

        let Some(missing_id) = constrain::external::is_valid_old_style(&names_id, declarations).collect(self, span)
        else {
            return;
        };
        for string_id in missing_id {
            let ty = QualifiedType::new(self.sema.types.int(), false, false);
            let name = Name::new(string_id, Span::default());
            let sym = Symbol::new(name, Some(ty), Some(Storage::Register), SymbolKind::Parameter, false);
            self.sema.declare(sym, &Span::default());
        }
    }

    fn add_parameter_declarator(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        decl: &DeclaratorNode,
        span: &Span,
    ) -> Option<SymbolId> {
        let qualif = constrain::declaration::resolve_type(&mut self.sema, ctx, specifiers, span);
        let (ty, decl) = self.sema.make_qualified_type(ctx, qualif, decl)?;
        let storage = constrain::declaration::get_storage(specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Register);
        constrain::external::param_storage_only_register(storage).collect(self, span)?;
        let name = decl.ident(ctx)?;
        let sym = Symbol::new(name, Some(ty), Some(storage), SymbolKind::Parameter, false);
        Some(self.sema.declare(sym, &decl.span))
    }
}

fn is_function_declarator(ctx: &Context, decl: &DeclaratorNode) -> bool {
    match decl.id.resolve(ctx) {
        Declarator::Function { declarator, .. } => matches!(declarator.id.resolve(ctx), Declarator::Ident(_)),
        _ => false,
    }
}

fn declares_tag(ctx: &Context, specifiers: &[DeclarationSpecifier]) -> bool {
    specifiers.iter().any(|specifier| match specifier {
        DeclarationSpecifier::Type(TypeSpecifier::Struct(id)) => id.resolve(ctx).name.is_some(),
        DeclarationSpecifier::Type(TypeSpecifier::Union(id)) => id.resolve(ctx).name.is_some(),
        DeclarationSpecifier::Type(TypeSpecifier::Enum(_)) => true,
        _ => false,
    })
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(parameters) = self.add_function(ctx, node) else { return };

        self.sema.scopes.push(ScopeKind::Prototype);
        let span = &parameters.span;
        match &parameters.param {
            FunctionParameters::Empty => self.param_empty(&node.old_style_declarations, span),
            FunctionParameters::ParameterTypeList(params) => {
                self.param_ident_style(ctx, params, &node.old_style_declarations, span)
            }
            FunctionParameters::OldStyle(names) => self.param_old_style(ctx, names, &node.old_style_declarations, span),
            FunctionParameters::Variadic(_) => unimplemented!(),
        }

        self.visit_compound_statement(ctx, &node.body);
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        if self.sema.scopes.kind() == ScopeKind::File {
            constrain::external::check_external_specifiers(specifiers).collect(self, span);
        }
        if node.init_declarators.is_empty() && !declares_tag(ctx, specifiers) {
            self.add_diag(Diag::only_diag(Diagnosis::EmptyDeclaration), span);
        }
        let declared_storage = constrain::declaration::get_storage(specifiers).collect(self, span);
        let qualif = constrain::declaration::resolve_type(&mut self.sema, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            let decl = &init_declarator.declarator;
            let Some((ty, decl)) = self.sema.make_qualified_type(ctx, qualif, decl) else { continue };
            let Some(name) = decl.ident(ctx) else { continue };
            if let Some(declared_storage) = declared_storage
                && declared_storage != Storage::Typedef
                && is_function_declarator(ctx, &decl)
            {
                constrain::declaration::extern_function_only(self.sema.scopes.kind(), declared_storage)
                    .collect(self, &decl.span);
            }
            let storage = declared_storage.unwrap_or(Storage::Auto);
            let kind = if storage == Storage::Typedef { SymbolKind::Typedef } else { SymbolKind::Variable };
            let is_init = init_declarator.initializer.is_some();
            self.sema
                .declare(Symbol::new(name, Some(ty), Some(storage), kind, is_init), &decl.span);
        }
        walk_declaration(self, ctx, node);
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        if let Labeled::Identifier(name, _) = node.inner {
            self.sema.add_label_symbol(name, &node.span, true);
        }
        walk_labeled_statement(self, ctx, node);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        if let JumpStatement::Goto(name) = node.stmt {
            self.sema.add_label_symbol(name, &node.span, false);
        }
        walk_jump_statement(self, ctx, node);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        match self.sema.scopes.kind() {
            ScopeKind::Prototype => self.sema.scopes.set_kind(ScopeKind::Function),
            _ => self.sema.scopes.push(ScopeKind::Block),
        }
        walk_compound_statement(self, ctx, node);
        self.sema.scopes.pop();
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        self.sema.visit_expression(ctx, node);
    }
}

pub struct Analyzer;
impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let sema = Self::resolve_names(&ctx);
        sema.into_context(ctx)
    }

    fn resolve_names(ctx: &Context) -> Sema {
        let mut resolver = SymbolResolver::new(ctx.target);
        resolver.sema.scopes.push(ScopeKind::File);
        walk_translation_unit(&mut resolver, ctx, &ctx.ast);
        resolver.sema.scopes.pop();
        assert!(resolver.sema.scopes.is_empty());
        resolver.sema
    }
}
