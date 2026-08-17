use std::collections::HashMap;

use crate::ast::{StringId, Type};
use crate::parser::Context;

#[derive(Default)]
struct Scope {
    symbols: HashMap<String, Symbol>,
}

struct Symbol {
    name: StringId,
    ty: Type,
}

pub struct Analyzer {
    ctx: Context,
    scopes: Vec<Scope>,
}

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let mut analyzer = Self {
            ctx,
            scopes: Vec::new(),
        };
        analyzer.collect_declarations();
        analyzer.resolve_names();
        analyzer.type_check();
        analyzer.ctx
    }

    fn collect_declarations(&mut self) {}

    fn resolve_names(&mut self) {}

    fn type_check(&mut self) {}
}
