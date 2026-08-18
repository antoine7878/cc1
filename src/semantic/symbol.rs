#![allow(unused)]
use std::collections::HashMap;

use crate::ast::visit::{Visitor, walk_declaration, walk_function_definition, walk_translation_unit};
use crate::ast::{DeclarationNode, FunctionDefinitionNode, StringId, Type};
use crate::parser::Context;

#[derive(Default)]
struct Scope {
    tags: HashMap<String, Symbol>,
    members: HashMap<String, Symbol>,
    labels: HashMap<String, Symbol>,
    ordinaries: HashMap<String, Symbol>,
}

struct Symbol {
    declaration: DeclarationNode,
    name: StringId,
    ty: Type,
}

#[derive(Default)]
struct NameResolver {
    scopes: Vec<Scope>,
}

impl Visitor for NameResolver {
    fn visit_declarator(&mut self, ctx: &Context, node: &crate::ast::DeclaratorNode, is_last: bool) {
        if let Some(name) = node.ident(ctx) {
            println!("{}", name.id.resolve(&ctx.arenas));
        }
    }
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let declarations = Self::resolve_names(&ctx);
        ctx
    }

    fn resolve_names(ctx: &Context) -> Vec<DeclarationNode> {
        let mut collector = NameResolver::default();
        walk_translation_unit(&mut collector, ctx, &ctx.ast, false);
        Vec::new()
    }
}
