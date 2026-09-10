use crate::codegen::llvm::{LlvmType, LlvmValue};
use crate::codegen::local::Local;
use std::fmt::{self, Display};
use std::io::Write;

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    counter: usize,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self { w, counter: 0 }
    }

    pub fn reset(&mut self, counter: usize) {
        self.counter = counter;
    }

    fn fresh(&mut self) -> LlvmValue {
        self.counter += 1;
        LlvmValue::SSA(self.counter)
    }

    fn line(&mut self, args: fmt::Arguments<'_>) {
        let _ = self.w.write_fmt(args);
        let _ = self.w.write_all(b"\n");
    }

    pub fn target(&mut self, datalayout: &str, triple: &str) {
        self.line(format_args!(r#"target datalayout = "{datalayout}""#));
        self.line(format_args!(r#"target triple = "{triple}""#));
    }

    pub fn blank(&mut self) {
        self.line(format_args!(""));
    }

    pub fn define(&mut self, ret: LlvmType, name: impl Display) {
        self.blank();
        self.line(format_args!("define {ret} @{name}() {{"));
    }

    pub fn end_function(&mut self) {
        self.line(format_args!("}}"));
    }

    pub fn alloca(&mut self, slot: Local, ty: LlvmType, align: u32) {
        self.line(format_args!("  {slot} = alloca {ty}, align {align}"));
    }

    pub fn load(&mut self, ty: LlvmType, slot: Local, align: u32) -> LlvmValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = load {ty}, ptr {slot}, align {align}"));
        r
    }

    pub fn binop(&mut self, op: &'static str, ty: LlvmType, a: LlvmValue, b: LlvmValue) -> LlvmValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {ty} {a}, {b}"));
        r
    }

    pub fn ret(&mut self, ty: LlvmType, v: LlvmValue) {
        self.line(format_args!("  ret {ty} {v}"));
    }

    pub fn ret_void(&mut self) {
        self.line(format_args!("  ret void"));
    }

    pub fn zext_bool(&mut self, v: LlvmValue, ty: LlvmType) -> LlvmValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = zext i1 {v} to {ty}"));
        r
    }
}
