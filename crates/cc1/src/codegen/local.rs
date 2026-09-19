use std::collections::HashMap;
use std::io::Write;
use std::iter;
use std::ops::Index;
use std::vec::IntoIter;

use crate::ast::visit::{walk_expression, walk_init_declarator};
use crate::ast::{Expression, ExpressionId, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, Visitor};
use crate::codegen::{Builder, LlvmName, LlvmParam, LlvmSymbol, LlvmType, ParamAttr, ReturnAttr, classify_param};
use crate::semantic::{DeclaredParams, Duration, FunctionHeader, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Locals {
    symbols: HashMap<SymbolId, LlvmSymbol>,
    pub order: Vec<SymbolId>,
    spills: HashMap<ExpressionId, LlvmSymbol>,
    spill_order: Vec<(ExpressionId, LlvmType)>,
    header: FunctionHeader,
    pub params: Vec<LlvmParam>,
    pub sret: Option<LlvmSymbol>,
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
        self.sret = None;
    }

    pub fn collect_locals(&mut self, node: &FunctionDefinitionNode) {
        self.collect_params(node);
        self.collect_declarations(node);
    }

    pub fn collect_params(&mut self, node: &FunctionDefinitionNode) {
        self.clear();
        self.header = sema().headers[&node.declarator.id].clone();
        let def = self.header.id.resolve();
        let mut next = 0;

        if let ReturnAttr::Sret { ty, align } = ReturnAttr::classify_return(def.return_ty) {
            next = 1;
            let slot = LlvmSymbol::ptr(LlvmName::SSA(0));
            self.sret = Some(slot);
            self.params.push(LlvmParam::new(slot, ParamAttr::SRet { ty, align }));
        }
        for &sym in &def.params {
            self.order.push(sym);
            let (ty, attr) = classify_param(sym.resolve().ty);
            let v = LlvmSymbol::new(ty, LlvmName::SSA(next));
            if matches!(attr, ParamAttr::ByVal { .. }) {
                self.symbols.insert(sym, v);
            }
            self.params.push(LlvmParam::new(v, attr));
            next += 1;
        }
    }

    pub fn collect_declarations(&mut self, node: &FunctionDefinitionNode) {
        self.visit_compound_statement(&node.body);
    }

    pub fn params(&self) -> &[LlvmParam] {
        &self.params
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self.header.params, DeclaredParams::Prototype { is_variadic: true, .. })
    }

    pub fn emit_decl<W: Write>(&mut self, builder: &mut Builder<W>) {
        for id in &self.order {
            if self.symbols.contains_key(id) {
                continue;
            }
            let qty = id.resolve().ty;
            let slot = builder.alloca(qty.llvm());
            self.symbols.insert(*id, slot);
        }

        for (id, ty) in &self.spill_order {
            let slot = builder.alloca(*ty);
            self.spills.insert(*id, slot);
        }

        let params = self.params.iter().skip(self.sret.is_some() as usize);
        for (param, id) in iter::zip(params, &self.order) {
            if let ParamAttr::Direct = param.attr {
                let local = self.symbols[id];
                builder.store(param.sym, local);
            }
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
