use std::collections::HashMap;
use std::io::Write;

use crate::ast::StringConstId;
use crate::codegen::llvm::{Builder, LlvmValue};
use crate::context::ctx;
use crate::semantic::sema;

#[derive(Debug, Default)]
pub struct Globals {
    map: HashMap<StringConstId, LlvmValue>,
}

impl Globals {
    pub fn get(&self, id: StringConstId) -> LlvmValue {
        self.map[&id]
    }

    pub fn emit<W: Write>(&mut self, b: &mut Builder<W>) {
        for (i, str) in ctx().arenas.strings.iter().enumerate() {
            let ty = if str.is_wide { sema().builtins.char } else { sema().builtins.int }.resolve();
            let lty = ty.llvm();
            let align = ctx().target.layout(ty).unwrap().align;
            let len = str.units.len();
            let v = b.string_literal(len, lty, str, align);
            self.map.insert(i.into(), v);
        }
    }
}
