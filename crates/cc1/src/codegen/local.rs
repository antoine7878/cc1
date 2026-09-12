use std::collections::HashMap;
use std::io::Write;
use std::ops::Index;
use std::vec::IntoIter;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::{Builder, LlvmSymbol};
use crate::semantic::{Duration, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Locals {
    map: HashMap<SymbolId, LlvmSymbol>,
    order: Vec<SymbolId>,
}

impl Index<SymbolId> for Locals {
    type Output = LlvmSymbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl Locals {
    pub fn get(&self, sym_id: SymbolId) -> Option<&LlvmSymbol> {
        self.map.get(&sym_id)
    }

    pub fn order_iter(&self) -> IntoIter<SymbolId> {
        self.order.clone().into_iter()
    }

    pub fn collect(&mut self, node: &FunctionDefinitionNode) {
        self.map.clear();
        self.order.clear();
        self.visit_compound_statement(&node.body);
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        for id in &self.order {
            let qty = id.resolve().ty.unwrap();
            let slot = b.alloca(qty.llvm());
            self.map.insert(*id, slot);
        }

        // for id in &self.order {
        //     let sym = id.resolve();
        //     let Some(init) = sym.initializer else { continue };
        //     let ty = id.resolve().ty.unwrap().llvm();
        //     match init.resolve() {
        //         Initializer::Zero => b.store(ty, LlvmValue::zero(), self.map[id]),
        //         Initializer::Value(v) => b.store(ty, v.llvm(), self.map[id]),
        //         Initializer::Address(_) => todo!("address init"),
        //         Initializer::String(s) => {
        //             let str = s.resolve();
        //         }
        //         Initializer::List(_) => todo!("list init"),
        //         Initializer::Expr(e) => match e.resolve() {
        //             Expression::Constant(e) => b.store(ty, e.value.llvm(), self.map[id]),
        //             e => todo!(),
        //         },
        //     }
        // }
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
