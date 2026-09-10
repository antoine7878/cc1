use crate::codegen::llvm::{LlvmType, LlvmValue};
use crate::codegen::local::Local;
use std::fmt::{self, Display};
use std::io::Write;

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    counter: usize,
    pub current_block: LlvmValue,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self {
            w,
            counter: 0,
            current_block: LlvmValue::SSA(0),
        }
    }

    pub fn reset(&mut self, counter: usize) {
        self.current_block = LlvmValue::SSA(0);
        self.counter = counter;
    }

    pub fn fresh(&mut self) -> LlvmValue {
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

    pub fn zext_bool(&mut self, v: LlvmValue, ty: LlvmType) -> LlvmValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = zext i1 {v} to {ty}"));
        r
    }

    pub fn ret(&mut self, ty: LlvmType, v: LlvmValue) {
        self.line(format_args!("  ret {ty} {v}"));
    }

    pub fn ret_void(&mut self) {
        self.line(format_args!("  ret void"));
    }

    pub fn br(&mut self, v: LlvmValue, l1: LlvmValue, l2: Option<LlvmValue>) {
        match l2 {
            Some(l2) => self.line(format_args!("  br i1 {v}, label {l1}, label {l2}")),
            None => self.line(format_args!("  br label {l1}")),
        }
    }

    pub fn named_label(&mut self, l: LlvmValue) {
        self.blank();
        self.current_block = l;
        match l {
            LlvmValue::SSA(i) => self.line(format_args!("{i}:")),
            LlvmValue::Lhs(i) => self.line(format_args!("lhs.l.{i}:")),
            LlvmValue::Rhs(i) => self.line(format_args!("rhs.l.{i}:")),
            _ => unimplemented!(),
        }
    }

    pub fn phi(&mut self, v1: LlvmValue, from1: LlvmValue, v2: LlvmValue, from2: LlvmValue) -> LlvmValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = phi i1 [ {v1}, {from1} ], [ {v2}, {from2} ]"));
        r
    }

    pub fn store(&mut self, ty: LlvmType, src: LlvmValue, dst: Local, align: u32) {
        self.line(format_args!("  store {ty} {src}, ptr {dst}, align {align}"))
    }
}
