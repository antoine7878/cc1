use crate::arena::ResolveWith;
use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_jump_statement, walk_labeled_statement,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode, Expression, ExpressionNode,
    FunctionDefinitionNode, InitDeclaratorNode, Initializer, InitializerNode, JumpStatement, JumpStatementNode,
    Labeled, LabeledStatementNode, Name, Storage, Tag, TypeSpecifier,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::expression;
use crate::semantic::{
    AssignmentContext, DeclaredParams, Definition, Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefId,
    ParamInfo, ParamTypes, QualifiedType, ResolvedType, ScopeKind, Sema, Symbol, SymbolId, SymbolKind, constrain,
    declaration, ice,
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

    fn add_function(
        &mut self,
        ctx: &Context,
        node: &FunctionDefinitionNode,
    ) -> Option<(FunctionDefId, DeclaredParams, Option<ParamTypes>)> {
        let span = &node.span;
        let decl_span = &node.declarator.span;

        let qualif = declaration::base_type(self.sema, ctx, &node.specifiers, span);
        let (rty, decl, params) = declaration::declared_function(self.sema, ctx, qualif, &node.declarator)?;
        let params = constrain::external::extract_function_declarator(params).collect(self, decl_span)?;

        let declared_storage = constrain::declaration::get_storage(&node.specifiers).collect(self, span);
        let storage = declared_storage.unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let name = decl.ident(ctx)?;
        let previous = self.sema.scopes.current(SymbolKind::Function, name.id);
        let declared = previous.and_then(|id| self.param_types(id));
        let ty = match &declared {
            Some(declared) if !matches!(params, DeclaredParams::Prototype { .. }) => {
                self.with_param_types(rty, declared.clone())
            }
            _ => rty,
        };
        match ty.id.resolve(self.sema) {
            &ResolvedType::Function { ret, .. } => self.return_ty = Some(ret),
            _ => unreachable!(),
        }
        if let Some(ret) = self.return_ty {
            constrain::external::check_definition_return(ret.is_void(self.sema) || ret.is_complete(self.sema), ret)
                .collect(self, decl_span);
        }
        let prior = self.sema.linkage_of_name(name.id);
        let linkage = Symbol::linkage_of(self.sema.scopes.kind(), declared_storage, SymbolKind::Function, prior);
        let mut sym = Symbol::function(name, ty, storage);
        sym.linkage = linkage;
        sym.definition = Definition::Definition;
        let sym = self.sema.declare(sym, decl_span);
        let declared = (previous == Some(sym)).then_some(declared).flatten();
        Some((self.sema.functions.declare(sym), params, declared))
    }

    fn param_types(&self, sym: SymbolId) -> Option<ParamTypes> {
        let ty = sym.resolve(self.sema).ty?;
        match ty.id.resolve(self.sema) {
            ResolvedType::Function { params, .. } => Some(params.clone()),
            _ => None,
        }
    }

    fn with_param_types(&mut self, ty: QualifiedType, params: ParamTypes) -> QualifiedType {
        let ret = match ty.id.resolve(self.sema) {
            ResolvedType::Function { ret, .. } => *ret,
            _ => return ty,
        };
        QualifiedType::new(self.sema.types.function(ret, params), ty.is_const, ty.is_volatile)
    }

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
            .filter_map(|sym| (*sym).resolve(self.sema).ty)
            .collect();
        if identifiers.len() != parameters.len() || declared.is_compatible_with_identifiers(self.sema, &identifiers) {
            return;
        }
        self.add_diag(
            Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Function, name)),
            span,
        );
    }

    fn param_empty(&mut self, lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
        if !lst.is_empty() {
            self.add_diag(Diag::err((), Diagnosis::ParameterTypeListWithList), span)
        }
        Vec::new()
    }

    fn param_prototype(&mut self, params: &[ParamInfo], lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
        if !constrain::external::is_valid_parameter_style(params, lst).collect(self, span) {
            return Vec::new();
        }
        if let [only] = params {
            let is_void = matches!(only.ty.id.resolve(self.sema), ResolvedType::Void);
            constrain::external::check_void_parameter(is_void).collect(self, &only.span);
        }
        for param in params {
            if param.ty.is_void(self.sema) {
                continue;
            }
            constrain::external::check_complete_parameter(param.ty.is_complete(self.sema), param.ty)
                .collect(self, &param.span);
        }
        params.iter().filter_map(|param| self.add_parameter(param)).collect()
    }

    fn add_parameter(&mut self, param: &ParamInfo) -> Option<SymbolId> {
        let name = param.name?;
        let storage = param.storage.unwrap_or(Storage::Auto);
        let sym = Symbol::parameter(name, param.ty, storage);
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
                                .map(|sym_id| sym_id.resolve(self.sema).name.id)
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
        let ty = QualifiedType::new(self.sema.builtins.int, false, false);
        missing_id
            .into_iter()
            .map(|string_id| Name::new(string_id, Span::default()))
            .map(|name| Symbol::parameter(name, ty, Storage::Auto))
            .for_each(|sym| {
                let _ = self.sema.declare(sym, &Span::default());
            });
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
        if !ty.is_void(self.sema) {
            constrain::external::check_complete_parameter(ty.is_complete(self.sema), ty).collect(self, &decl.span);
        }
        let storage = declared_storage.unwrap_or(Storage::Auto);
        let sym = Symbol::parameter(name, ty, storage);
        Some(self.sema.declare(sym, &decl.span))
    }

    fn requires_complete_object(&self, ty: QualifiedType, storage: Storage, is_init: bool) -> bool {
        if ty.is_void(self.sema) {
            return true;
        }
        let is_unsized_array = matches!(ty.id.resolve(self.sema), ResolvedType::Array { len: None, .. });
        if is_init && is_unsized_array {
            return false;
        }
        if is_init {
            return true;
        }
        matches!(self.sema.scopes.kind(), ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern
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

impl SymbolResolver<'_> {
    fn resolve_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.sema.binding_seen(node.id) {
            return;
        }
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            let sym = self.sema.scopes.lookup_ordinary(name.id);
            if sym.is_none() {
                self.add_diag(Diag::err((), Diagnosis::UndeclaredIdentifier(*name)), &node.span);
            }
            self.sema.set_binding(node.id, sym);
        }
        expression::resolve_expression(self.sema, ctx, node);
    }

    fn walk(self, ty: ResolvedType, cursor: bool) {
        let target = match ty {
            ResolvedType::Tag(tag_id) if matches!(tag_id.resolve(self.sema).kind, Tag::Struct | Tag::Union) => (),
            ResolvedType::Array { elem, len } => elem,
            _ => [ty],
        };
        for t in target {
            if 
        }
    }

    // walk(ty, cursor):
    // if ty is struct/union: targets = members (union: first member only)
    // elif ty is array:      targets = elements
    // else:                  targets = [ty]           # scalar
    // for each target:
    //     if next initializer is a brace list -> recurse into it, must not overflow
    //     else -> consume flat items from the cursor (brace elision)
    // leftover initializers -> ExcessInitializers      # 6.5.7 "no more initializers than objects"

    fn resolve_initializer(&mut self, ctx: &Context, ty: QualifiedType, node: &InitializerNode) {
        match &node.init {
            Initializer::Single(e) => {
                self.visit_expression(ctx, e);
                if let Err(inner) = expression::init(self.sema, ctx, ty, e, AssignmentContext::Initialization) {
                    self.add_diag(Diag::err((), inner), &e.span);
                }
            }
            Initializer::List(inits) => {
                let (elem, len) = match ty.id.resolve(self.sema) {
                    &ResolvedType::Array { elem, len } => (elem, len),
                    _ => (ty, Some(1)),
                };
                for (i, node) in inits.iter().enumerate() {
                    if len.is_some_and(|l| l <= i) {
                        self.add_diag(Diag::err((), Diagnosis::ArrayInitTooLong), &inits[i].span);
                        break;
                    }
                    self.resolve_initializer(ctx, elem, node);
                }
            }
        }
    }

    fn resolve_return(&mut self, ctx: &Context, node: &JumpStatementNode, return_ty: QualifiedType) {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => {
                self.visit_expression(ctx, e);
                if let Err(inner) = expression::init(self.sema, ctx, return_ty, e, AssignmentContext::Return) {
                    self.add_diag(Diag::err((), inner), &e.span);
                }
            }
            JumpStatement::Return(None) if return_ty.id != self.sema.builtins.void => {
                self.add_diag(Diag::err((), Diagnosis::InvalidReturnType), &node.span);
            }
            _ => (),
        }
    }

    fn implicit_declare_function(&mut self, fn_name: &Name, span: &Span) {
        let ret = QualifiedType::new(self.sema.builtins.int, false, false);
        let fn_ty = self.sema.types.function(ret, ParamTypes::Unspecified);
        let ty = QualifiedType::new(fn_ty, false, false);
        let sym = Symbol::function(*fn_name, ty, Storage::Extern);
        self.sema.declare(sym, span);
    }
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
        self.return_ty = None;
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        if self.sema.scopes.kind() == ScopeKind::File {
            constrain::external::check_external_specifiers(specifiers).collect(self, span);
        }
        if node.init_declarators.is_empty() && !declares_tag(ctx, specifiers) {
            self.add_diag(Diag::err((), Diagnosis::EmptyDeclaration), span);
        }
        let declared_storage = constrain::declaration::get_storage(specifiers).collect(self, span);
        let qualif = declaration::base_type(self.sema, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            let decl = &init_declarator.declarator;
            let diag_count_before = self.sema.diagnosis.len();
            let Some((ty, decl)) = declaration::declared_type(self.sema, ctx, qualif, decl) else { continue };
            let already_diagnosed = self.sema.diagnosis.len() != diag_count_before;
            let Some(name) = decl.ident(ctx) else { continue };
            let is_function = matches!(ty.id.resolve(self.sema), ResolvedType::Function { .. });
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
            if kind == SymbolKind::Variable && !already_diagnosed && self.requires_complete_object(ty, storage, is_init)
            {
                constrain::declaration::check_complete_object(ty.is_complete(self.sema), ty).collect(self, &decl.span);
            }
            let prior = self.sema.linkage_of_name(name.id);
            let scope_kind = self.sema.scopes.kind();
            let mut sym = Symbol::new(name, Some(ty), Some(storage), kind, is_init);
            sym.linkage = Symbol::linkage_of(scope_kind, declared_storage, kind, prior);
            sym.duration = Symbol::duration_of(scope_kind, declared_storage, kind);
            sym.definition = Symbol::definition_of(scope_kind, declared_storage, is_init, kind);
            let sym_id = self.sema.declare(sym, &decl.span);
            self.sema.declarations.insert(init_declarator.declarator.id, sym_id);
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
        match &node.stmt {
            JumpStatement::Goto(name) => self.sema.add_label_symbol(*name, &node.span, false),
            JumpStatement::Return(e) => {
                if let Some(e) = e {
                    self.visit_expression(ctx, e);
                }
                if let Some(return_ty) = self.return_ty {
                    self.resolve_return(ctx, node, return_ty);
                }
            }
            _ => walk_jump_statement(self, ctx, node),
        }
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
        if let Expression::FunctionCall(f, _) = node.id.resolve(ctx)
            && let Expression::Identifier(fn_name) = f.id.resolve(ctx)
            && self.sema.scopes.lookup_ordinary(fn_name.id).is_none()
        {
            self.implicit_declare_function(fn_name, &node.span);
        }
        walk_expression(self, ctx, node);
        self.resolve_expression(ctx, node);
    }

    fn visit_init_declarator(&mut self, ctx: &Context, node: &InitDeclaratorNode) {
        let decl = &node.declarator;
        let Some(&sym) = self.sema.declarations.get(&decl.id) else { return };
        let Some(ty) = sym.resolve(self.sema).ty else { return };
        self.visit_declarator(ctx, &node.declarator);
        if let Some(init) = &node.initializer {
            self.resolve_initializer(ctx, ty, init);
        }
    }
}
