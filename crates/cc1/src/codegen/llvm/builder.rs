use std::fmt::{self, Display};
use std::io::Write;
use std::iter::once;

use crate::codegen::{LlvmInit, LlvmName, LlvmSymbol, LlvmType};
use crate::ast::{StringConstant, Tag};
use crate::semantic::{Definition, Linkage, ParamTypes, ResolvedType, SymbolId, TagDef, TagDefId, sema};

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    counter: usize,
    pub current_block: LlvmName,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self { w, counter: 0, current_block: LlvmName::SSA(0) }
    }

    pub fn reset(&mut self, counter: usize) {
        self.current_block = LlvmName::SSA(0);
        self.counter = counter;
    }

    pub fn fresh(&mut self) -> LlvmName {
        self.counter += 1;
        LlvmName::SSA(self.counter)
    }

    fn write_all(&mut self, str: &[u8]) {
        let _ = self.w.write_all(str);
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

    pub fn define(&mut self, binding: LlvmSymbol, parameters: &[LlvmSymbol], is_variadic: bool) {
        self.blank();
        let _ = self.w.write_fmt(format_args!("define {}", binding));
        self.params(parameters, is_variadic);
        self.line(format_args!(" {{"));
    }

    fn params(&mut self, parameters: &[LlvmSymbol], is_variadic: bool) {
        let mut it = parameters.iter().peekable();
        let _ = self.w.write_all(b"(");
        while let Some(param) = it.next() {
            let _ = self.w.write_fmt(format_args!("{param}"));
            if it.peek().is_some() {
                let _ = self.w.write_all(b", ");
            }
        }
        if is_variadic {
            let _ = self.w.write_all(b", ...");
        }
        let _ = self.w.write_all(b")");
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

    pub fn call(&mut self, f: LlvmSymbol, parameters: &[LlvmSymbol]) -> LlvmSymbol {
        let r = self.fresh();
        let _ = match f.ty.is_void() {
            true => self.w.write_fmt(format_args!("  call {f}")),
            false => self.w.write_fmt(format_args!("  {r} = call {f}")),
        };
        self.params(parameters, false);
        self.line(format_args!(""));
        LlvmSymbol::new(f.ty, r)
    }

    pub fn gep(&mut self, elem: LlvmType, base: LlvmSymbol, idx: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.line(format_args!("  {r} = getelementptr inbounds {elem}, ptr {}, {idx}", base.name));
        LlvmSymbol::ptr(r)
    }

    pub fn string_literal(&mut self, name: LlvmName, str: &StringConstant) {
        let len = str.units.len() + 1;
        let ty = if str.is_wide { LlvmType::int() } else { LlvmType::char() };
        let _ = self.w.write_fmt(format_args!("{name} = private unnamed_addr constant "));
        self.array(ty, str.units.iter().chain(once(&0)), len);
        self.blank();
    }

    fn array<T, I>(&mut self, ty: LlvmType, elems: I, len: usize)
    where
        I: IntoIterator<Item = T>,
        T: Display,
    {
        let _ = self.w.write_fmt(format_args!("[{} x {ty}] [", len));
        for (i, c) in elems.into_iter().enumerate() {
            let _ = self.w.write_fmt(format_args!("{ty} {c}"));
            if i != len - 1 {
                let _ = self.w.write_all(b", ");
            }
        }
        let _ = self.w.write_all(b"]");
    }

    pub fn type_def(&mut self, id: TagDefId) {
        let def = id.resolve();
        let ty = LlvmType::Tag(id);
        match def.kind {
            Tag::Enum => (),
            _ if !def.is_complete => self.line(format_args!("{ty} = type opaque")),
            Tag::Struct => self.struct_def(ty, def),
            Tag::Union => self.union_def(ty, def),
        }
    }

    fn struct_def(&mut self, ty: LlvmType, def: &TagDef) {
        let _ = self.w.write_fmt(format_args!("{ty} = type {{ "));
        let mut it = def.members.iter().peekable();
        while let Some(member) = it.next() {
            let sym = member.sym.expect("bitfield in a type definition");
            let _ = self.w.write_fmt(format_args!("{}", sym.resolve().ty.llvm()));
            if it.peek().is_some() {
                self.write_all(b", ");
            }
        }
        self.line(format_args!(" }}"));
    }

    fn union_def(&mut self, ty: LlvmType, def: &TagDef) {
        let size = ty.size();
        let members = def.members.iter().filter_map(|member| member.sym).map(|sym| sym.resolve().ty);
        let Some(widest) = members.max_by_key(|qty| sema().layout(&qty.id).align) else {
            return self.line(format_args!("{ty} = type {{ [{size} x {}] }}", LlvmType::char()));
        };
        match size - sema().layout(&widest.id).size {
            0 => self.line(format_args!("{ty} = type {{ {} }}", widest.llvm())),
            pad => self.line(format_args!("{ty} = type {{ {}, [{pad} x {}] }}", widest.llvm(), LlvmType::char())),
        }
    }

    pub fn declare(&mut self, sym_id: SymbolId) {
        let sym = sym_id.resolve();
        let ResolvedType::Function { ret, params } = sym.ty.id.resolve() else {
            unreachable!("declare on a non-function")
        };
        let _ = self.w.write_fmt(format_args!("declare {} {}(", ret.llvm(), LlvmName::Global(sym_id)));
        match params {
            ParamTypes::Unspecified => self.write_all(b"..."),
            ParamTypes::Prototype { params, is_variadic } => {
                let mut it = params.iter().peekable();
                while let Some(param) = it.next() {
                    let _ = self.w.write_fmt(format_args!("{}", param.llvm()));
                    if it.peek().is_some() {
                        self.write_all(b", ");
                    }
                }
                if *is_variadic {
                    self.write_all(if params.is_empty() { b"..." } else { b", ..." });
                }
            }
        }
        self.line(format_args!(")"));
    }

    pub fn global(&mut self, sym_id: SymbolId) {
        let sym = sym_id.resolve();
        let name = LlvmName::Global(sym_id);
        let align = sema().layout(&sym.ty.id).align;
        let kind = if sym.ty.is_const { "constant" } else { "global" };
        if sym.definition == Definition::Declaration {
            return self.line(format_args!("{name} = external {kind} {}, align {align}", sym.ty.llvm()));
        }
        let linkage = match sym.linkage {
            Linkage::External => "",
            Linkage::Internal | Linkage::None => "internal ",
        };
        let init = LlvmInit::new(sym.ty, sym.initializer.map(|i| i.resolve()));
        self.line(format_args!("{name} = {linkage}{kind} {init}, align {align}"));
    }
}
