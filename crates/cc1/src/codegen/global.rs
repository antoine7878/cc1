use std::collections::HashMap;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{InitDeclaratorNode, StringConstId, TranslationUnitNode, Visitor};
use crate::codegen::llvm::LlvmName;
use crate::codegen::{LlvmSymbol, LlvmType};
use crate::context::ctx;
use crate::semantic::{Duration, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Globals {
    pub strings: HashMap<StringConstId, LlvmSymbol>,
    map: HashMap<SymbolId, LlvmSymbol>,
    pub order: Vec<SymbolId>,
}

impl Globals {
    pub fn get_symbol(&self, id: SymbolId) -> Option<&LlvmSymbol> {
        self.map.get(&id)
    }

    pub fn get_literal(&self, id: StringConstId) -> Option<&LlvmSymbol> {
        self.strings.get(&id)
    }

    pub fn collect(&mut self, node: &TranslationUnitNode) {
        self.collect_literals();
        self.collect_externals();
        self.visit_translation_unit(node);
        self.order.sort_by_key(|id| usize::from(*id));
    }

    fn collect_externals(&mut self) {
        for ext in sema().externals.values() {
            self.register(ext.symbol);
        }
    }

    fn register(&mut self, sym_id: SymbolId) {
        if self.map.contains_key(&sym_id) {
            return;
        }
        let qty = sym_id.resolve().ty;
        let name = LlvmName::Global(sym_id);
        if qty.is_function(sema()) {
            self.map.insert(sym_id, LlvmSymbol::new(qty.llvm(), name));
            return;
        }
        self.map.insert(sym_id, LlvmSymbol::ptr(name));
        self.order.push(sym_id);
    }

    fn collect_literals(&mut self) {
        for (i, str) in ctx().arenas.strings.iter().enumerate() {
            let ty = if str.is_wide { LlvmType::int() } else { LlvmType::char() };
            let v = LlvmSymbol::new(ty, LlvmName::StringLiteral(i.into()));
            self.strings.insert(i.into(), v);
        }
    }
}

impl Visitor for Globals {
    fn visit_init_declarator(&mut self, node: &InitDeclaratorNode) {
        walk_init_declarator(self, node);
        let sym_id = sema().declarations[&node.declarator.id];
        if sym_id.resolve().duration == Duration::Static {
            self.register(sym_id);
        }
    }
}
