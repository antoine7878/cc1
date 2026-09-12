use std::collections::HashMap;
use std::io::Write;

use crate::ast::{StringConstId, StringId};
use crate::codegen::LlvmSymbol;
use crate::codegen::llvm::{Builder, LlvmName};
use crate::context::ctx;
use crate::semantic::{ResolvedType, sema};

#[derive(Debug, Default)]
pub struct Globals {
    strings: HashMap<StringConstId, LlvmSymbol>,
    functions: HashMap<StringId, LlvmSymbol>,
}

impl Globals {
    pub fn get_function(&self, id: StringId) -> Option<&LlvmSymbol> {
        self.functions.get(&id)
    }

    pub fn get_literal(&self, id: StringConstId) -> Option<&LlvmSymbol> {
        self.strings.get(&id)
    }

    pub fn emit_literals<W: Write>(&mut self, b: &mut Builder<W>) {
        for (i, str) in ctx().arenas.strings.iter().enumerate() {
            let v = b.string_literal(str);
            self.strings.insert(i.into(), v);
        }
    }

    pub fn register_functions(&mut self) {
        for (a, b) in sema().externals.iter() {
            let ResolvedType::Function { ret, .. } = b.symbol.resolve().ty.unwrap().id.resolve() else {
                unreachable!()
            };
            let s = LlvmSymbol::new(ret.llvm(), LlvmName::Global(b.symbol));
            self.functions.insert(*a, s);
        }
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        self.register_functions();
        self.emit_literals(b);
    }
}
