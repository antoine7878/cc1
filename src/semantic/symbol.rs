use std::collections::HashMap;

use crate::ast::{DeclarationNode, StringId, Type};
use crate::parser::Context;

#[derive(Default)]
struct Scope {
    symbols: HashMap<String, Symbol>,
}

struct Symbol {
    declaration: DeclarationNode,
    name: StringId,
    ty: Type,
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let declarations = Self::collect_declarations(&ctx);
        let names = Self::resolve_names(&ctx, &declarations);
        let types = Self::resolve_types(&ctx, &declarations);
        ctx
    }

    fn collect_declarations(ctx: &Context) -> Vec<DeclarationNode> {
        let ret = Vec::new();
        println!("collect_declarations");
        ret
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

    // fn type_check() {}
}
