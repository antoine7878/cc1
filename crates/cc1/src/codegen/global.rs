use std::collections::HashMap;

use crate::ast::visit::walk_init_declarator;
use crate::ast::{InitDeclaratorNode, StringConstId, TranslationUnitNode, Visitor};
use crate::codegen::{LlvmName, LlvmSymbol};
use crate::context::ctx;
use crate::semantic::{Duration, Initializer, SymbolId, sema};

#[derive(Debug, Default)]
pub struct Globals {
    pub strings: HashMap<StringConstId, LlvmSymbol>,
    pub lists: HashMap<SymbolId, LlvmSymbol>,
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
        self.collect_list_init();
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
            self.map.insert(sym_id, LlvmSymbol::ptr(name));
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

    fn collect_list_init(&mut self) {
        for i in 0..sema().symbols.len() {
            let sym_id: SymbolId = i.into();
            let sym = sym_id.resolve();
            if sym.duration != Duration::Automatic {
                continue;
            }
            let Some(init) = sym.initializer else { continue };
            if !matches!(init.resolve(), Initializer::List(_) | Initializer::String(_)) {
                continue;
            }
            self.lists.insert(sym_id, LlvmSymbol::ptr(LlvmName::ListInit(sym_id)));
        }
    }
}

impl Visitor for Globals {
    fn visit_init_declarator(&mut self, node: &InitDeclaratorNode) {
        walk_init_declarator(self, node);
        let Some(&sym_id) = sema().declarations.get(&node.declarator.id) else { return };
        if sym_id.resolve().duration == Duration::Static {
            self.register(sym_id);
        }
    }
}
