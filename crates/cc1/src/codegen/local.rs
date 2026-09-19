use std::collections::HashMap;
use std::io::Write;
use std::iter;
use std::ops::Index;
use std::vec::IntoIter;

use crate::ast::visit::{walk_expression, walk_init_declarator};
use crate::ast::{Expression, ExpressionId, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::{Builder, LlvmName, LlvmSymbol, LlvmType};
use crate::semantic::{DeclaredParams, Duration, FunctionHeader, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Locals {
    symbols: HashMap<SymbolId, LlvmSymbol>,
    pub order: Vec<SymbolId>,
    spills: HashMap<ExpressionId, LlvmSymbol>,
    spill_order: Vec<(ExpressionId, LlvmType)>,
    header: FunctionHeader,
    pub params: Vec<LlvmSymbol>,
}

impl Index<SymbolId> for Locals {
    type Output = LlvmSymbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl Locals {
    pub fn get(&self, sym_id: SymbolId) -> Option<&LlvmSymbol> {
        self.symbols.get(&sym_id)
    }

    pub fn order_iter(&self) -> IntoIter<SymbolId> {
        self.order.clone().into_iter()
    }

    pub fn spill(&self, id: ExpressionId) -> LlvmSymbol {
        self.spills[&id]
    }

    fn clear(&mut self) {
        self.symbols.clear();
        self.order.clear();
        self.spills.clear();
        self.spill_order.clear();
        self.params.clear();
    }

    pub fn collect_locals(&mut self, node: &FunctionDefinitionNode) {
        self.collect_params(node);
        self.collect_declarations(node);
    }

    pub fn collect_params(&mut self, node: &FunctionDefinitionNode) {
        self.clear();
        self.header = sema().headers[&node.declarator.id].clone();
        for (i, param) in self.header.id.resolve().params.iter().enumerate() {
            self.order.push(*param);
            let v = LlvmSymbol::new(param.resolve().ty.llvm(), LlvmName::SSA(i));
            self.params.push(v);
        }
    }

    pub fn collect_declarations(&mut self, node: &FunctionDefinitionNode) {
        self.visit_compound_statement(&node.body);
    }

    pub fn params(&self) -> &[LlvmSymbol] {
        &self.params
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self.header.params, DeclaredParams::Prototype { is_variadic: true, .. })
    }

    pub fn emit_decl<W: Write>(&mut self, builder: &mut Builder<W>) {
        for id in &self.order {
            let qty = id.resolve().ty;
            let slot = builder.alloca(qty.llvm());
            self.symbols.insert(*id, slot);
        }

        for (id, ty) in &self.spill_order {
            let slot = builder.alloca(*ty);
            self.spills.insert(*id, slot);
        }

        for (param, id) in iter::zip(&self.params, &self.order) {
            let local = self.symbols[id];
            builder.store(*param, local);
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

    fn visit_expression(&mut self, node: &ExpressionNode) {
        walk_expression(self, node);
        let Expression::FunctionCall(_, _) = node.id.resolve() else { return };
        let Some(re) = sema().expressions.get(node.id) else { return };
        if !re.ty.is_record(sema()) {
            return;
        }
        self.spill_order.push((node.id, re.ty.llvm()));
    }
}
