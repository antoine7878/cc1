#![allow(unused)]
use std::collections::HashMap;

use crate::ast::visit::{Visitor, walk_compound_statement, walk_declarator, walk_expression, walk_translation_unit};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, Expression,
    ExpressionNode, FunctionDefinitionNode, Name, Qualifier, Storage, StringId, StructId, Type, TypeSpecifier, UnionId,
};
use crate::define_arena;
use crate::parser::YYToken::specifier_qualifier_list;
use crate::parser::{Context, Span};
use crate::semantic::declarations::{extern_function_only, get_qualifier, get_storage};
use crate::semantic::{Diag, Diagnosis, ResolvedType, TypeSpecifierCounter, declarations, diagnosis};

define_arena!(Symbol, SymbolArena, SymbolId, symbols);

impl SymbolArena {
    pub fn add(
        &mut self,
        name: Name,
        ty: ResolvedType,
        storage: Storage,
        is_const: bool,
        is_volatile: bool,
        declarator: DeclaratorNode,
    ) -> SymbolId {
        self.alloc(Symbol {
            name,
            ty,
            storage,
            is_const,
            is_volatile,
            declarator,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<String, Symbol>,
    members: HashMap<String, Symbol>,
    labels: HashMap<String, Symbol>,
    ordinaries: HashMap<StringId, Symbol>,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Symbol {
    name: Name,
    ty: ResolvedType,
    storage: Storage,
    declarator: DeclaratorNode,
    is_const: bool,
    is_volatile: bool,
}

#[derive(Default, Debug)]
struct SymbolResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<Diagnosis>,
}

impl SymbolResolver {
    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn extract_diag<T>(&mut self, diag: Diag<T>, span: &Span) -> T {
        let Diag::<T> { res, diagnosis } = diag;
        if let Some(d) = diagnosis {
            self.diagnosis.push(Diagnosis::new(d, *span));
        }
        res
    }

    fn scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn get_type(&mut self, specifiers: &[DeclarationSpecifier], name: &Name, span: &Span) -> Option<ResolvedType> {
        let d = declarations::resolve_type(specifiers);
        self.extract_diag(d, span)
    }

    fn find_symbol(&mut self, ctx: &Context, name: &Name) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.ordinaries.contains_key(&name.id))
    }

    // fn check_symbol(&mut self, ctx: &Context, name: &Name, span: &Span) {
    //     if !self.find_symbol(ctx, name) {
    //         self.diagnosis.push(Diagnosis::undeclared_identifier(name, span))
    //     }
    // }
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode, is_last: bool) {
        let Some(name) = node.declarator.ident(ctx) else { return };
        let span = &node.span;
        let storage = self
            .extract_diag(get_storage(&node.specifiers), span)
            .unwrap_or(Storage::Extern);
        self.extract_diag(extern_function_only(self.scope().ty, storage), span);
        let ty = self.get_type(&node.specifiers, &name, span);
        let (is_const, is_volatile) = self.extract_diag(get_qualifier(&node.specifiers), span);

        if let Some(ty) = ty {
            ctx.arenas
                .symbols
                .add(name, ty, storage, is_const, is_volatile, node.declarator.clone());
        }
    }

    // #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    // pub struct Symbol {
    //     name: StringId,
    //     ty: Type,
    //     storage: Storage,
    //     declaration: DeclaratorNode,
    //     is_const: bool,
    //     is_volatile: bool,
    // }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode, is_last: bool) {
        if let Expression::Identifier(name) = node.id.resolve(&ctx.arenas) {
            // self.check_symbol(ctx, name, node.span);
        }
        walk_expression(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        self.scopes.push(Scope::new(ScopeType::Block));
        walk_compound_statement(self, ctx, node, is_last);
        self.scopes.pop();
        for diag in &self.diagnosis {
            diag.print(ctx);
        }
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
        Vec::new()
    }
}
