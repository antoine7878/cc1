use std::collections::HashMap;
use std::io::Write;
use std::iter;
use std::ops::Index;
use std::vec::IntoIter;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::{Builder, LlvmName, LlvmSymbol, LlvmType};
use crate::semantic::{Duration, FunctionDefId, SymbolId, sema};

#[derive(Debug)]
pub struct Locals {
    map: HashMap<SymbolId, LlvmSymbol>,
    order: Vec<SymbolId>,
    f: FunctionDefId,
    pub parameters: Vec<LlvmSymbol>,
}

impl Default for Locals {
    fn default() -> Self {
        Self { map: HashMap::default(), order: Vec::default(), f: 0.into(), parameters: Vec::default() }
    }
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

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.parameters.clear();
    }

    pub fn collect(&mut self, node: &FunctionDefinitionNode) {
        self.clear();
        self.f = sema().function_defs[&node.declarator.id];
        for (i, param) in self.f.resolve().parameters.iter().enumerate() {
            self.order.push(*param);
            let v = LlvmSymbol::new(param.resolve().ty.llvm(), LlvmName::SSA(i));
            self.parameters.push(v);
        }
        self.visit_compound_statement(&node.body);
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        for id in &self.order {
            let qty = id.resolve().ty;
            let slot = b.alloca(qty.llvm());
            self.map.insert(*id, slot);
        }
        for (param, id) in iter::zip(&self.parameters, &self.order) {
            let local = self.map[id];
            b.store(*param, local);
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
