#![allow(unused)]
use std::collections::HashMap;

use crate::ast::visit::{Visitor, walk_translation_unit};
use crate::ast::{DeclarationNode, FunctionDefinitionNode, StringId, Type};
use crate::parser::{Arenas, Context};

#[derive(Default)]
struct Scope {
    symbols: HashMap<String, Symbol>,
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
    fn visit_declaration(&mut self, _arenas: &Arenas, node: &DeclarationNode, _is_last: bool) {
        // self.declarations.push(node.clone());
    }

    // fn visit_function_definition(&mut self, _arenas: &Arenas, _node: &FunctionDefinitionNode, _is_last: bool) {}
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let declarations = Self::collect_declarations(&ctx);
        let _names = Self::resolve_names(&ctx, &declarations);
        let _types = Self::resolve_types(&ctx, &declarations);
        ctx
    }

    fn collect_declarations(ctx: &Context) -> Vec<DeclarationNode> {
        let mut collector = NameResolver::default();
        println!("collect_declarations");
        walk_translation_unit(&mut collector, &ctx.arenas, &ctx.ast, false);
        collector.declarations
    }

    fn resolve_names(ctx: &Context, declarations: &[DeclarationNode]) -> Vec<StringId> {
        let ret = Vec::new();
        println!("resolve_names");
        ret
    }

    fn resolve_types(ctx: &Context, declarations: &[DeclarationNode]) -> Vec<Type> {
        let ret = Vec::new();
        println!("resolve_types");
        ret
    }
}
