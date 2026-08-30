use crate::arena::ResolveWith;
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_jump_statement, walk_labeled_statement,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode, Expression, ExpressionNode,
    FunctionDefinitionNode, Initializer, JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name,
    Storage, TypeSpecifier,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::expression;
use crate::semantic::{
    DeclaredParams, Diag, DiagCollector, Diagnosis, DiagnosisNode, ExpressionKind, FunctionDefId, ParamInfo,
    ParamTypes, QualifiedType, ResolvedExpression, ResolvedType, ScopeKind, Sema, Symbol, SymbolId, SymbolKind,
    constrain, declaration, ice,
};

#[derive(Debug)]
pub struct SymbolResolver<'a> {
    pub sema: &'a mut Sema,
}

impl DiagCollector for SymbolResolver<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.sema.diagnosis
    }
}

impl<'a> SymbolResolver<'a> {
    pub fn new(sema: &'a mut Sema) -> Self {
        Self { sema }
    }

    fn add_function(
        &mut self,
        ctx: &Context,
        node: &FunctionDefinitionNode,
    ) -> Option<(FunctionDefId, DeclaredParams, Option<ParamTypes>)> {
        let span = &node.span;
        let decl_span = &node.declarator.span;

        let qualif = declaration::base_type(self.sema, ctx, &node.specifiers, span);
        let (ty, decl, params) = declaration::declared_function(self.sema, ctx, qualif, &node.declarator)?;
        let params = constrain::external::extract_function_declarator(params).collect(self, decl_span)?;

        let storage = constrain::declaration::get_storage(&node.specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let name = decl.ident(ctx)?;
        let previous = self.sema.scopes.current(SymbolKind::Function, name.id);
        let declared = previous.and_then(|id| self.param_types(id));
        // 6.5.4.3 If one type has a parameter type list and the other type is specified by a function
        // definition that contains a (possibly empty) identifier list: the parameter half of that
        // comparison needs the identifiers, so only the return type is compared here.
        let ty = match &declared {
            Some(declared) if !matches!(params, DeclaredParams::Prototype { .. }) => {
                self.with_param_types(ty, declared.clone())
            }
            _ => ty,
        };
        let sym = Symbol::new(name, Some(ty), Some(storage), SymbolKind::Function, true);
        let sym = self.sema.declare(sym, decl_span);
        let declared = (previous == Some(sym)).then_some(declared).flatten();
        Some((self.sema.functions.declare(sym), params, declared))
    }

    fn param_types(&self, sym: SymbolId) -> Option<ParamTypes> {
        let ty = sym.resolve(&*self.sema).ty?;
        match ty.id.resolve(&*self.sema) {
            ResolvedType::Function { params, .. } => Some(params.clone()),
            _ => None,
        }
    }

    fn with_param_types(&mut self, ty: QualifiedType, params: ParamTypes) -> QualifiedType {
        let ret = match ty.id.resolve(&*self.sema) {
            ResolvedType::Function { ret, .. } => *ret,
            _ => return ty,
        };
        QualifiedType::new(self.sema.types.function(ret, params), ty.is_const, ty.is_volatile)
    }

    // 6.5.4.3 If one type has a parameter type list and the other type is specified by a function
    // definition that contains a (possibly empty) identifier list, both shall agree in the number of
    // parameters, and the type of each prototype parameter shall be compatible with the type that results
    // from the application of the default argument promotions to the type of the corresponding identifier.
    fn check_identifier_list(
        &mut self,
        declared: &ParamTypes,
        params: &DeclaredParams,
        parameters: &[SymbolId],
        name: Name,
        span: &Span,
    ) {
        if let DeclaredParams::Names(names) = params
            && names.len() != parameters.len()
        {
            return;
        }
        let identifiers: Vec<QualifiedType> = parameters
            .iter()
            .filter_map(|sym| (*sym).resolve(&*self.sema).ty)
            .collect();
        if identifiers.len() != parameters.len() || declared.is_compatible_with_identifiers(self.sema, &identifiers) {
            return;
        }
        self.add_diag(
            Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Function, name)),
            span,
        );
    }

    fn param_empty(&mut self, lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
        if !lst.is_empty() {
            self.add_diag(Diag::only_diag(Diagnosis::ParameterTypeListWithList), span)
        }
        Vec::new()
    }

    fn param_prototype(&mut self, params: &[ParamInfo], lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
        if !constrain::external::is_valid_parameter_style(params, lst).collect(self, span) {
            return Vec::new();
        }
        if let [only] = params {
            let is_void = matches!(only.ty.id.resolve(&*self.sema), ResolvedType::Void);
            constrain::external::check_void_parameter(is_void).collect(self, &only.span);
        }
        params.iter().filter_map(|param| self.add_parameter(param)).collect()
    }

    fn add_parameter(&mut self, param: &ParamInfo) -> Option<SymbolId> {
        let name = param.name?;
        let storage = param.storage.unwrap_or(Storage::Auto);
        let sym = Symbol::new(name, Some(param.ty), Some(storage), SymbolKind::Parameter, false);
        Some(self.sema.declare(sym, &name.span))
    }

    fn param_old_style(
        &mut self,
        ctx: &Context,
        names: &[Name],
        lst: &[DeclarationNode],
        span: &Span,
    ) -> Vec<SymbolId> {
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
                                .map(|sym_id| sym_id.resolve(&*self.sema).name.id)
                        })
                        .collect::<Vec<_>>()
                },
            )
            .collect();

        let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

        let Some(missing_id) = constrain::external::is_valid_old_style(&names_id, declarations).collect(self, span)
        else {
            return Vec::new();
        };
        for string_id in missing_id {
            let ty = QualifiedType::new(self.sema.builtins.int, false, false);
            let name = Name::new(string_id, Span::default());
            let sym = Symbol::new(name, Some(ty), Some(Storage::Auto), SymbolKind::Parameter, false);
            self.sema.declare(sym, &Span::default());
        }
        names
            .iter()
            .filter_map(|name| self.sema.scopes.lookup_ordinary(name.id))
            .collect()
    }

    fn add_parameter_declarator(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        decl: &DeclaratorNode,
        span: &Span,
    ) -> Option<SymbolId> {
        let qualif = declaration::base_type(self.sema, ctx, specifiers, span);
        let (ty, decl) = declaration::declared_type(self.sema, ctx, qualif, decl)?;
        let ty = self.sema.types.adjust_parameter(ty);
        let declared_storage = constrain::declaration::get_storage(specifiers).collect(self, span);
        if let Some(storage) = declared_storage {
            constrain::external::param_storage_only_register(storage).collect(self, span)?;
        }
        let name = decl.ident(ctx)?;
        let storage = declared_storage.unwrap_or(Storage::Auto);
        let sym = Symbol::new(name, Some(ty), Some(storage), SymbolKind::Parameter, false);
        Some(self.sema.declare(sym, &decl.span))
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

impl Visitor for Sema {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        walk_expression(self, ctx, node);
        if self.bindings.contains_key(&node.id) {
            return;
        }
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            let sym = self.scopes.lookup_ordinary(name.id);
            if sym.is_none() {
                self.add_diag(Diag::only_diag(Diagnosis::UndeclaredIdentifier(*name)), &node.span);
            }
            self.bindings.insert(node.id, sym);
        }
        expression::run(self, ctx, node);
    }

    // fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
    //     let specifiers = &node.specifiers;
    //     let span = &node.span;
    //     let declared_storage = constrain::declaration::get_storage(specifiers).collect(self, span);
    //     let Some(ty) = declaration::base_type(self, ctx, specifiers, span) else { return };
    //     for init_decl in &node.init_declarators {
    //         let Some(init_node) = &init_decl.initializer else { continue };
    //         match &init_node.init {
    //             Initializer::Single(e) => {
    //                 self.visit_expression(ctx, e);
    //                 expression::init(self, ctx, ty, e);
    //             }
    //             Initializer::List(_) => todo!(),
    //         }
    //     }
    // }
}

impl Visitor for SymbolResolver<'_> {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some((def, params, declared)) = self.add_function(ctx, node) else { return };

        self.sema.scopes.push(ScopeKind::Prototype);
        let lst = &node.old_style_declarations;
        let span = &node.declarator.span;
        let parameters = match &params {
            DeclaredParams::Unspecified => self.param_empty(lst, span),
            DeclaredParams::Names(names) => self.param_old_style(ctx, names, lst, span),
            DeclaredParams::Prototype { params, .. } => self.param_prototype(params, lst, span),
        };
        if let Some(declared) = declared
            && !matches!(params, DeclaredParams::Prototype { .. })
            && let Some(name) = node.declarator.ident(ctx)
        {
            self.check_identifier_list(&declared, &params, &parameters, name, span);
        }
        self.sema.functions.complete(def, parameters);

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
        let qualif = declaration::base_type(self.sema, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            let decl = &init_declarator.declarator;
            let Some((ty, decl)) = declaration::declared_type(self.sema, ctx, qualif, decl) else { continue };
            let Some(name) = decl.ident(ctx) else { continue };
            let is_function = matches!(ty.id.resolve(&*self.sema), ResolvedType::Function { .. });
            if let Some(declared_storage) = declared_storage
                && declared_storage != Storage::Typedef
                && is_function
            {
                constrain::declaration::extern_function_only(self.sema.scopes.kind(), declared_storage)
                    .collect(self, &decl.span);
            }
            let default_storage = if is_function { Storage::Extern } else { Storage::Auto };
            let storage = declared_storage.unwrap_or(default_storage);
            let kind = match storage {
                Storage::Typedef => SymbolKind::Typedef,
                _ if is_function => SymbolKind::Function,
                _ => SymbolKind::Variable,
            };
            let is_init = init_declarator.initializer.is_some();
            self.sema
                .declare(Symbol::new(name, Some(ty), Some(storage), kind, is_init), &decl.span);
        }
        walk_declaration(self, ctx, node);
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        match &node.inner {
            Labeled::Identifier(name, _) => self.sema.add_label_symbol(*name, &node.span, true),
            Labeled::Case(expr, _) => {
                ice::eval_constant(self.sema, ctx, expr);
            }
            Labeled::Default(_) => (),
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
