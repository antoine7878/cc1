use std::fmt::{self};
use std::io::Write;

use crate::ast::StringConstant;
use crate::codegen::LlvmSymbol;
use crate::codegen::llvm::{LlvmName, LlvmType};

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    counter: usize,
    pub current_block: LlvmName,
    str_counter: usize,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self { w, counter: 0, str_counter: 0, current_block: LlvmName::SSA(0) }
    }

    pub fn reset(&mut self, counter: usize) {
        self.current_block = LlvmName::SSA(0);
        self.counter = counter;
    }

    pub fn fresh_string(&mut self) -> LlvmName {
        self.str_counter += 1;
        LlvmName::StringLiteral(self.str_counter)
    }

    pub fn fresh(&mut self) -> LlvmName {
        self.counter += 1;
        LlvmName::SSA(self.counter)
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

    pub fn define(&mut self, binding: LlvmSymbol) {
        self.blank();
        self.line(format_args!("define {}() {{", binding));
    }

    pub fn end_function(&mut self) {
        self.line(format_args!("}}"));
    }

    pub fn alloca(&mut self, ty: LlvmType) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = alloca {ty}"));
        LlvmSymbol::ptr(r)
    }

    pub fn load(&mut self, ty: LlvmType, slot: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = load {ty}, {slot}"));
        LlvmSymbol::new(ty, r)
    }

    pub fn store(&mut self, src: LlvmSymbol, dst: LlvmSymbol) {
        self.line(format_args!("  store {src}, {dst}"))
    }

    pub fn binop(&mut self, op: &'static str, lhs: LlvmSymbol, rhs: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {lhs}, {}", rhs.name));
        LlvmSymbol::new(lhs.ty, r)
    }

    pub fn unop(&mut self, op: &'static str, v: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {v}"));
        LlvmSymbol::new(v.ty, r)
    }

    pub fn cmp(&mut self, op: &'static str, lhs: LlvmSymbol, rhs: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {lhs}, {}", rhs.name));
        LlvmSymbol::new(LlvmType::bool(), r)
    }

    pub fn convert(&mut self, conv: &'static str, from: LlvmSymbol, to: LlvmType) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = {conv} {from} to {to}"));
        LlvmSymbol::new(to, r)
    }

    pub fn ret(&mut self, b: LlvmSymbol) {
        self.line(format_args!("  ret {b}"));
    }

    pub fn ret_void(&mut self) {
        self.line(format_args!("  ret void"));
    }

    pub fn br(&mut self, cond: LlvmSymbol, l1: LlvmName, l2: Option<LlvmName>) {
        match l2 {
            Some(l2) => self.line(format_args!("  br {cond}, label {l1}, label {l2}")),
            None => self.line(format_args!("  br label {l1}")),
        }
    }

    pub fn named_label(&mut self, l: LlvmName) {
        self.blank();
        self.current_block = l;
        match l {
            LlvmName::SSA(i) => self.line(format_args!("{i}:")),
            LlvmName::Label(i, j) => self.line(format_args!("l.{i}.{j}:")),
            _ => unimplemented!(),
        }
    }

    pub fn phi(&mut self, s1: LlvmSymbol, l1: LlvmName, s2: LlvmSymbol, l2: LlvmName) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = phi {} [ {}, {l1} ], [ {}, {l2} ]", s1.ty, s1.name, s2.name));
        LlvmSymbol::new(s1.ty, r)
    }

    pub fn string_literal(&mut self, str: &StringConstant) -> LlvmSymbol {
        let s = self.fresh_string();
        let len = str.units.len() + 1;
        let ty = if str.is_wide { LlvmType::int() } else { LlvmType::char() };
        let _ = self.w.write_fmt(format_args!("{s} = private unnamed_addr constant [{len} x {ty}] ["));
        for c in &str.units {
            let _ = self.w.write_fmt(format_args!("{ty} {c}, "));
        }
        self.line(format_args!("{ty} 0]"));
        LlvmSymbol::new(ty, s)
    }

    pub fn call(&mut self, f: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = call {f}()"));
        LlvmSymbol::new(f.ty, r)
    }

    pub fn gep(&mut self, elem: LlvmType, base: LlvmSymbol, idx: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = getelementptr inbounds {elem}, ptr {}, {idx}", base.name));
        LlvmSymbol::ptr(r)
    }
}

// pub struct LlvmListInitilizer<'a> {
//     values: &'a [LlvmValue],
//     ty: LlvmType,
// }

// impl<'a> Display for LlvmListInitilizer<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.values.iter().try_for_each(|v| write!(f, "{}, {}", v, self.ty))
//     }
// }
