use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::io::Write;

use crate::ast::visit::walk_declarator;
use crate::ast::{DeclaratorNode, FunctionDefinitionNode, StringId, Visitor};
use crate::codegen::llvm::Builder;
use crate::context::Context;
use crate::semantic::{Duration, SymbolId};

#[derive(Debug, Clone, Copy)]
pub struct Local {
    pub name: StringId,
    pub dup: u32,
}

impl Local {
    pub fn llvm<'a>(self, ctx: &'a Context) -> LocalName<'a> {
        LocalName { local: self, ctx }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalName<'a> {
    local: Local,
    ctx: &'a Context,
}

impl Display for LocalName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = self.local.name.resolve(self.ctx);
        match self.local.dup {
            0 => write!(f, "%{name}"),
            dup => write!(f, "%{name}.{dup}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Locals {
    map: HashMap<SymbolId, Local>,
    order: Vec<SymbolId>,
    seen: HashMap<StringId, u32>,
}

impl Locals {
    pub fn collect(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        self.map.clear();
        self.order.clear();
        self.seen.clear();
        self.visit_compound_statement(ctx, &node.body);
    }

    pub fn get(&self, sym_id: SymbolId) -> Local {
        self.map[&sym_id]
    }

    pub fn emit<W: Write>(&self, ctx: &Context, b: &mut Builder<W>) {
        for &sym_id in &self.order {
            let qty = sym_id.resolve(ctx).ty.unwrap();
            let local = self.map[&sym_id];
            b.alloca(local.llvm(ctx), qty.llvm(ctx), qty.layout(ctx).unwrap().align);
        }
    }
}

impl Visitor for Locals {
    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode) {
        walk_declarator(self, ctx, node);
        let sym_id = ctx.sema.declarations[&node.id];
        let sym = sym_id.resolve(ctx);
        if sym.duration != Duration::Automatic {
            return;
        }
        let dup = self.seen.entry(sym.name.id).or_insert(0);
        self.map.insert(sym_id, Local { name: sym.name.id, dup: *dup });
        self.order.push(sym_id);
        *dup += 1;
    }
}
