use std::collections::HashMap;

use crate::ast::visit::walk_declarator;
use crate::ast::{DeclaratorNode, FunctionDefinitionNode, Visitor};
use crate::context::Context;
use crate::semantic::{Duration, SymbolId};

#[derive(Debug)]
pub struct AllocaCollector<'a> {
    locals: &'a mut HashMap<SymbolId, usize>,
    counter: usize,
}

impl<'a> AllocaCollector<'a> {
    pub fn run(locals: &'a mut HashMap<SymbolId, usize>, ctx: &Context, node: &FunctionDefinitionNode) {
        let mut collector = AllocaCollector { locals, counter: 1 };
        collector.visit_function_definition(ctx, node);
    }
}

impl<'a> Visitor for AllocaCollector<'a> {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        self.visit_compound_statement(ctx, &node.body);
    }

    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode) {
        walk_declarator(self, ctx, node);
        let sym_id = ctx.sema.declarations[&node.id];
        let sym = sym_id.resolve(ctx);
        if sym.duration != Duration::Automatic {
            return;
        }
        self.locals.insert(sym_id, self.counter);
        self.counter += 1;
    }
}
