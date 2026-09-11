use std::collections::HashMap;
use std::io::Write;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::llvm::{Builder, LlvmValue};
use crate::semantic::{Duration, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Locals {
    map: HashMap<SymbolId, LlvmValue>,
    order: Vec<SymbolId>,
}

impl Locals {
    pub fn collect(&mut self, node: &FunctionDefinitionNode) {
        self.map.clear();
        self.order.clear();
        self.visit_compound_statement(&node.body);
    }

    pub fn get(&self, sym_id: SymbolId) -> Option<&LlvmValue> {
        self.map.get(&sym_id)
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        for &sym_id in &self.order {
            let qty = sym_id.resolve().ty.unwrap();
            let slot = b.alloca(qty.llvm(), qty.layout().unwrap().align);
            self.map.insert(sym_id, slot);
        }
    }
}

impl Visitor for Locals {
    fn visit_init_declarator(&mut self, node: &InitDeclaratorNode) {
        walk_init_declarator(self, node);
        let sym_id = sema().declarations[&node.declarator.id];
        let sym = sym_id.resolve();
        if sym.duration != Duration::Automatic {
            return;
        }
        self.order.push(sym_id);
    }
}
