use std::collections::HashMap;
use std::io::Write;
use std::iter;
use std::ops::Index;
use std::vec::IntoIter;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::{Builder, LlvmName, LlvmSymbol};
use crate::semantic::{DeclaredParams, Duration, FunctionHeader, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Locals {
    map: HashMap<SymbolId, LlvmSymbol>,
    pub order: Vec<SymbolId>,
    f: FunctionHeader,
    pub parameters: Vec<LlvmSymbol>,
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

    pub fn collect_locals(&mut self, node: &FunctionDefinitionNode) {
        self.collect_params(node);
        self.collect_declarations(node);
    }

    pub fn collect_params(&mut self, node: &FunctionDefinitionNode) {
        self.clear();
        self.f = sema().function_defs[&node.declarator.id].clone();
        for (i, param) in self.f.id.resolve().parameters.iter().enumerate() {
            self.order.push(*param);
            let v = LlvmSymbol::new(param.resolve().ty.llvm(), LlvmName::SSA(i));
            self.parameters.push(v);
        }
    }

    pub fn collect_declarations(&mut self, node: &FunctionDefinitionNode) {
        self.visit_compound_statement(&node.body);
    }

    pub fn parameters(&self) -> &[LlvmSymbol] {
        &self.parameters
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self.f.params, DeclaredParams::Prototype { is_variadic: true, .. })
    }

    pub fn emit_decl<W: Write>(&mut self, b: &mut Builder<W>) {
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
        let Some(&sym_id) = sema().declarations.get(&node.declarator.id) else { return };
        let sym = sym_id.resolve();
        if sym.duration != Duration::Automatic {
            return;
        }
        self.order.push(sym_id);
    }
}
