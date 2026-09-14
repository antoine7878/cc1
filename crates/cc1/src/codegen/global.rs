use std::collections::HashMap;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{InitDeclaratorNode, StringConstId, TranslationUnitNode, Visitor};
use crate::codegen::llvm::LlvmName;
use crate::codegen::LlvmSymbol;
use crate::context::ctx;
use crate::semantic::{Duration, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Globals {
    pub strings: HashMap<StringConstId, LlvmSymbol>,
    map: HashMap<SymbolId, LlvmSymbol>,
    pub order: Vec<SymbolId>,
    pub functions: Vec<SymbolId>,
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
        self.functions.sort_by_key(|id| usize::from(*id));
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
            self.functions.push(sym_id);
            return;
        }
        self.map.insert(sym_id, LlvmSymbol::ptr(name));
        self.order.push(sym_id);
    }

    fn collect_literals(&mut self) {
        for i in 0..ctx().arenas.strings.len() {
            self.strings.insert(i.into(), LlvmSymbol::ptr(LlvmName::StringLiteral(i.into())));
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
