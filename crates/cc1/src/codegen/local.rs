use std::collections::HashMap;
use std::io::Write;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{Expression, FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::llvm::{Builder, LlvmValue};
use crate::semantic::{Duration, Initializer, SymbolId, sema};

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
        for id in &self.order {
            let qty = id.resolve().ty.unwrap();
            let slot = b.alloca(qty.llvm());
            self.map.insert(*id, slot);
        }

        for id in &self.order {
            let sym = id.resolve();
            let Some(init) = sym.initializer else { continue };
            let ty = id.resolve().ty.unwrap().llvm();
            match init.resolve() {
                Initializer::Zero => b.store(ty, LlvmValue::zero(), self.map[id]),
                Initializer::Value(v) => b.store(ty, v.llvm(), self.map[id]),
                Initializer::Address(_) => todo!("address init"),
                Initializer::String(_) => todo!("string init"),
                Initializer::List(_) => todo!("list init"),
                Initializer::Expr(e) => match e.resolve() {
                    Expression::Constant(_) => {
                        let v = sema().expr_consts[*e];
                        b.store(ty, v.llvm(), self.map[id])
                    }
                    e => todo!(),
                },
            }
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
