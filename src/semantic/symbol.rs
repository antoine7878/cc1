#![allow(unused)]
use std::collections::HashMap;

use crate::ast::visit::{Visitor, walk_compound_statement, walk_declarator, walk_expression, walk_translation_unit};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, Declarator, DeclaratorNode, Expression, ExpressionNode, Name, StringId,
};
use crate::define_arena;
use crate::parser::{Context, Span};
use crate::semantic::Diagnosis;

// define_arena!(Symbol, SymbolArena, SymbolId, symbols);

#[derive(Default, Debug)]
struct Scope {
    tags: HashMap<String, Symbol>,
    members: HashMap<String, Symbol>,
    labels: HashMap<String, Symbol>,
    ordinaries: HashMap<StringId, Symbol>,
}

#[derive(Debug)]
struct Symbol {
    declaration: DeclaratorNode,
    name: StringId,
}

impl Symbol {
    fn new(declaration: DeclaratorNode, name: StringId) -> Self {
        Self { declaration, name }
    }
}

#[derive(Default, Debug)]
struct NameResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<Diagnosis>,
}

impl NameResolver {
    fn add_symbol(&mut self, ctx: &Context, node: &DeclaratorNode) {
        if let Some(name) = node.ident(ctx) {
            self.scopes
                .last_mut()
                .unwrap()
                .ordinaries
                .insert(name.id, Symbol::new(node.clone(), name.id));
        }
    }

    fn find_symbol(&mut self, ctx: &Context, name: &Name) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.ordinaries.contains_key(&name.id))
    }

    fn check_symbol(&mut self, ctx: &Context, name: &Name, span: Span) {
        if !self.find_symbol(ctx, name) {
            self.diagnosis
                .push(Diagnosis::undeclared_identifier(name.clone(), span))
        }
    }
}

impl Visitor for NameResolver {
    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode, is_last: bool) {
        self.add_symbol(ctx, node);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode, is_last: bool) {
        match node.id.resolve(&ctx.arenas) {
            Expression::Identifier(name) => self.check_symbol(ctx, name, node.span),
            Expression::StringLiteral(name) => self.check_symbol(ctx, name, node.span),
            _ => (),
        }
        walk_expression(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        self.scopes.push(Scope::default());
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
        let mut collector = NameResolver::default();
        collector.scopes.push(Scope::default());
        walk_translation_unit(&mut collector, ctx, &ctx.ast, false);
        collector.scopes.pop();
        assert!(collector.scopes.is_empty());
        Vec::new()
    }
}
