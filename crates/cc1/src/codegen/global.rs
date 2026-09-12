use std::collections::HashMap;
use std::io::Write;

use crate::ast::{StringConstId, StringId};
use crate::codegen::llvm::{Builder, LlvmValue};
use crate::context::ctx;
use crate::semantic::sema;

#[derive(Debug, Default)]
pub struct Globals {
    strings: HashMap<StringConstId, LlvmValue>,
    functions: HashMap<StringId, LlvmValue>,
}

impl Globals {
    pub fn get_function(&self, id: StringId) -> Option<&LlvmValue> {
        self.functions.get(&id)
    }
    pub fn get_literal(&self, id: StringConstId) -> Option<&LlvmValue> {
        self.strings.get(&id)
    }

    pub fn emit_literals<W: Write>(&mut self, b: &mut Builder<W>) {
        for (i, str) in ctx().arenas.strings.iter().enumerate() {
            let ty = str.ty(&sema().builtins).resolve();
            let lty = ty.llvm();
            let len = str.units.len();
            let v = b.string_literal(len, lty, str);
            self.strings.insert(i.into(), v);
        }
    }

    pub fn register_functions(&mut self) {
        for (a, b) in sema().externals.iter() {
            self.functions.insert(*a, LlvmValue::Global(b.symbol));
        }
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        self.register_functions();
        self.emit_literals(b);
    }
}
