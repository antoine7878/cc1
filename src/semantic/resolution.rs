#![allow(unused)]
use std::collections::{HashMap, HashSet};

use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_labeled_statement, walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, ExpressionNode,
    ExternalDeclaration, FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, Labeled, Name,
    ParameterDeclaration, declaration,
};
use crate::ast::{Expression, Storage, StringId};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ResolvedTypeArena, SymbolArena, SymbolId, SymbolKind, constrain,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<String, SymbolId>,
    members: HashMap<String, SymbolId>,
    labels: HashMap<String, SymbolId>,
    ordinaries: HashMap<StringId, SymbolId>,
    ty: ScopeKind,
}

impl Scope {
    pub fn new(ty: ScopeKind) -> Self {
        Self {
            tags: HashMap::new(),
            members: HashMap::new(),
            labels: HashMap::new(),
            ordinaries: HashMap::new(),
            ty,
        }
    }
}

#[derive(Default, Debug)]
struct SymbolResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<DiagnosisNode>,
    symbols: SymbolArena,
    types: ResolvedTypeArena,
}

impl DiagCollector for SymbolResolver {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl SymbolResolver {
    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn make_qualified_type(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        decl: &DeclaratorNode,
        span: &Span,
    ) -> Option<(QualifiedType, DeclaratorNode)> {
        let (is_const, is_volatile) = constrain::declaration::get_qualifier(specifiers).collect(self, span);
        let ty = constrain::declaration::resolve_type(specifiers).collect(self, span)?;
        let id = self.types.alloc(ty);
        let inner_most = QualifiedType::new(id, is_const, is_volatile);
        Some(self.extract_pointer(ctx, decl, inner_most))
    }

    fn extract_pointer(
        &mut self,
        ctx: &Context,
        declarator: &DeclaratorNode,
        inner_most: QualifiedType,
    ) -> (QualifiedType, DeclaratorNode) {
        match declarator.id.resolve(ctx) {
            Declarator::Pointer { qualifiers, inner } => {
                let (qty, decl) = self.extract_pointer(ctx, inner, inner_most);
                let (is_const, is_volatile) =
                    constrain::declaration::check_qualifier(qualifiers).collect(self, &declarator.span);
                let id = self.types.pointer(qty);
                (QualifiedType::new(id, is_const, is_volatile), decl)
            }
            _ => (inner_most, declarator.clone()),
        }
    }

    fn add_symbol(
        &mut self,
        name: Name,
        ty: QualifiedType,
        storage: Storage,
        kind: SymbolKind,
        declarator: Option<DeclaratorNode>,
    ) -> SymbolId {
        let sym_id = self.symbols.add(name, ty, storage, kind);
        self.scope_mut().ordinaries.insert(name.id, sym_id);
        sym_id
    }

    fn add_function<'a>(
        &mut self,
        ctx: &'a Context,
        node: &FunctionDefinitionNode,
    ) -> Option<&'a FunctionParametersNode> {
        let span = &node.span;

        let (ty, fn_decl) = self.make_qualified_type(ctx, &node.specifiers, &node.declarator, span)?;
        let (decl, parameters) =
            constrain::external::extract_function_declarator(fn_decl.id.resolve(ctx)).collect(self, span)?;

        let storage = constrain::declaration::get_storage(&node.specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let name = fn_decl.ident(ctx)?;
        self.add_symbol(name, ty, storage, SymbolKind::Function, Some(node.declarator.clone()));
        Some(parameters)
    }

    fn param_empty(&mut self, lst: &[DeclarationNode], span: &Span) {
        if !lst.is_empty() {
            self.diagnosis
                .push(DiagnosisNode::new(Diagnosis::ParameterTypeListWithList, *span))
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
        if names
            .iter()
            .any(|name| !constrain::external::check_typedef(ctx, name).collect(self, span))
        {
            return;
        }
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
                                .map(|sym_id| self.symbols.get(sym_id).name.id)
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
            let ty_id = self.types.int();
            self.add_symbol(
                Name::new(string_id, Span::default()),
                QualifiedType::new(ty_id, false, false),
                Storage::Register,
                SymbolKind::Parameter,
                None,
            );
        }
    }

    fn add_parameter_declarator(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        decl: &DeclaratorNode,
        span: &Span,
    ) -> Option<SymbolId> {
        let (ty, decl) = self.make_qualified_type(ctx, specifiers, decl, span)?;
        let storage = constrain::declaration::get_storage(specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Register);
        constrain::external::param_storage_only_register(storage).collect(self, span)?;
        let name = decl.ident(ctx)?;
        Some(self.add_symbol(name, ty, storage, SymbolKind::Parameter, Some(decl)))
    }
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode, _is_last: bool) {
        let Some(parameters) = self.add_function(ctx, node) else { return };

        self.scopes.push(Scope::new(ScopeKind::Prototype));
        let span = &parameters.span;
        match &parameters.param {
            FunctionParameters::Empty => self.param_empty(&node.old_style_declarations, span),
            FunctionParameters::ParameterTypeList(params) => {
                self.param_ident_style(ctx, params, &node.old_style_declarations, span)
            }
            FunctionParameters::OldStyle(names) => self.param_old_style(ctx, names, &node.old_style_declarations, span),
            FunctionParameters::Variadic(_) => unimplemented!(),
        }

        self.visit_compound_statement(ctx, &node.body, true);
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode, is_last: bool) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        for init_declarator in &node.init_declarators {
            let decl = &init_declarator.declarator;
            let Some((ty, decl)) = self.make_qualified_type(ctx, specifiers, decl, span) else { continue };
            let Some(name) = decl.ident(ctx) else { continue };
            let storage = constrain::declaration::get_storage(&node.specifiers)
                .collect(self, span)
                .unwrap_or(Storage::Auto);
            self.add_symbol(name, ty, storage, SymbolKind::Variable, Some(decl));
        }
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &crate::ast::LabeledStatementNode, is_last: bool) {
        match &node.inner {
            Labeled::Identifier(name, stmt) => {
                let ty = self.types.label();
                let _ = self.add_symbol(
                    *name,
                    QualifiedType::new(ty, false, false),
                    Storage::Static,
                    SymbolKind::Label,
                    None,
                );
            }
            Labeled::Case(expr, stmt) => (),
            Labeled::Default(stmt) => (),
        }
        walk_labeled_statement(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        match self.scopes.last().unwrap().ty {
            ScopeKind::Prototype => self.scopes.last_mut().unwrap().ty = ScopeKind::Block,
            _ => self.scopes.push(Scope::new(ScopeKind::Block)),
        }
        walk_compound_statement(self, ctx, node, is_last);
        self.scopes.pop();
    }
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        Self::resolve_names(&ctx);
        ctx
    }

    fn resolve_names(ctx: &Context) -> Vec<DeclarationNode> {
        let mut collector = SymbolResolver::default();
        collector.scopes.push(Scope::new(ScopeKind::File));
        walk_translation_unit(&mut collector, ctx, &ctx.ast, false);
        collector.scopes.pop();
        assert!(collector.scopes.is_empty());
        for symbol in &collector.symbols.data {
            println!("{}", symbol.name.id.resolve(ctx));
        }
        for diag in &collector.diagnosis {
            let _ = diag.print(ctx);
        }
        Vec::new()
    }
}
