#![allow(unused)]
use std::collections::HashMap;

use crate::ast::declaration::DeclaratorId;
use crate::ast::visit::{Visitor, walk_compound_statement, walk_expression, walk_translation_unit};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, ExpressionNode,
    FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, Name,
};
use crate::ast::{Expression, Storage, StringId};
use crate::parser::{Context, Span};
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ResolvedTypeArena, ResolvedTypeId, SymbolArena, SymbolId,
    constrain,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
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
    ty: ScopeType,
}

impl Scope {
    pub fn new(ty: ScopeType) -> Self {
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

    // fn find_symbol(&mut self, name: &Name) -> bool {
    //     self.scopes
    //         .iter()
    //         .rev()
    //         .any(|scope| scope.ordinaries.contains_key(&name.id))
    // }

    // fn check_symbol(&mut self, ctx: &Context, name: &Name, span: &Span) {
    //     if !self.find_symbol(ctx, name) {
    //         self.diagnosis.push(Diagnosis::undeclared_identifier(name, span))
    //     }
    // }

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

    // fn make_qualified_type(&mut self, specifiers: &[DeclarationSpecifier], span: &Span) -> Option<QualifiedType> {
    //     let (is_const, is_volatile) = self.extract_diag(constrain::declaration::get_qualifier(specifiers), span);
    //     let ty = self.extract_diag(constrain::declaration::resolve_type(specifiers), span)?;
    //     let id = self.types.alloc(ty);
    //     Some(QualifiedType::new(id, is_const, is_volatile))
    // }

    // fn extract_function(
    //     &mut self,
    //     ctx: &Context,
    //     declarator: &DeclaratorNode,
    //     inner_most: QualifiedType,
    // ) -> (QualifiedType, DeclaratorNode, FunctionParametersNode) {
    //     match declarator.id.resolve(ctx) {
    //         Declarator::Pointer { qualifiers, inner } => {
    //             let (qty, decl, params) = self.extract_function(ctx, inner, inner_most);
    //             let (is_const, is_volatile) =
    //                 self.extract_diag(constrain::declaration::check_qualifier(qualifiers), &declarator.span);
    //             let id = self.types.pointer(qty);
    //             (QualifiedType::new(id, is_const, is_volatile), decl, params)
    //         }
    //         Declarator::Function { declarator, params } => (inner_most, declarator.clone(), params.clone()),
    //         _ => unreachable!(),
    //     }
    // }

    fn add_function(&mut self, ctx: &Context, node: &FunctionDefinitionNode) -> Option<FunctionParametersNode> {
        let span = &node.span;

        let (ty, fn_decl) = self.make_qualified_type(ctx, &node.specifiers, &node.declarator, span)?;
        let a = constrain::external::extract_function_declarator(fn_decl).collect(self, span);

        // let parial_type = self.make_qualified_type(&node.specifiers, span)?;
        // let (ty, fn_declarator, parameters) = self.extract_function(ctx, &node.declarator, parial_type);
        let name = fn_decl.ident(ctx)?;

        let storage = constrain::declaration::get_storage(&node.specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);

        constrain::declaration::extern_function_only(self.scope().ty, storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let sym_id = self.symbols.add(name, ty, storage, node.declarator.clone());
        self.scope_mut().ordinaries.insert(name.id, sym_id);
        Some(parameters)
    }

    fn add_arguments(&mut self, ctx: &Context, node: &DeclarationNode) {
        let span = &node.span;
        let qualified_type = self.make_qualified_type(&node.specifiers, span);
    }
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode, _is_last: bool) {
        let Some(args) = self.add_function(ctx, node) else { return };

        self.scopes.push(Scope::new(ScopeType::Prototype));

        self.scopes.pop();

        self.visit_compound_statement(ctx, &node.body, true);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode, is_last: bool) {
        if let Expression::Identifier(_name) = node.id.resolve(ctx) {
            // self.check_symbol(ctx, name, node.span);
        }
        walk_expression(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        self.scopes.push(Scope::new(ScopeType::Block));
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
        collector.scopes.push(Scope::new(ScopeType::File));
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
