use std::fmt::{self, Display};
use std::io::Write;
use std::iter::once;

use crate::ast::{StringConstant, Tag};
use crate::codegen::{
    LlvmInit, LlvmName, LlvmParam, LlvmSymbol, LlvmType, ParamAttr, ReturnAttr, classify_param, struct_elements,
};
use crate::semantic::{
    DefinitionState, Initializer, Linkage, QualifiedType, ResolvedType, SymbolId, TagDef, TagDefId, sema,
};
use crate::target::Layout;

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    ssa_counter: usize,
    label_counter: usize,
    pub current_block: LlvmName,
    pub has_block_ret: bool,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self { w, ssa_counter: 0, label_counter: 0, current_block: LlvmName::SSA(0), has_block_ret: false }
    }

    pub fn reset(&mut self, counter: usize) {
        self.has_block_ret = false;
        self.current_block = LlvmName::SSA(0);
        self.ssa_counter = counter;
        self.label_counter = 0;
    }

    pub fn fresh(&mut self) -> LlvmName {
        self.ssa_counter += 1;
        LlvmName::SSA(self.ssa_counter)
    }

    pub fn fresh_label(&mut self) -> LlvmName {
        self.label_counter += 1;
        LlvmName::label(self.label_counter)
    }

    fn write_str(&mut self, str: &str) {
        if !self.has_block_ret {
            let _ = self.w.write_all(str.as_bytes());
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) {
        if !self.has_block_ret {
            let _ = self.w.write_fmt(args);
        }
    }

    fn write_line(&mut self, args: fmt::Arguments<'_>) {
        self.write_fmt(args);
        self.blank();
    }

    pub fn target(&mut self, datalayout: &str, triple: &str) {
        self.write_line(format_args!(r#"target datalayout = "{datalayout}""#));
        self.write_line(format_args!(r#"target triple = "{triple}""#));
    }

    pub fn blank(&mut self) {
        self.write_str("\n");
    }

    pub fn define_function(&mut self, sym_id: SymbolId, params: &[LlvmParam], is_variadic: bool) {
        let ResolvedType::Function { ret, .. } = sym_id.resolve().ty.id.resolve() else {
            unreachable!("define on a non-function")
        };
        let ret_attr = ReturnAttr::classify_return(*ret);
        self.blank();
        self.reset(params.len());
        self.write_fmt(format_args!("define {} {}", ret_attr.ret_llvm(), LlvmName::Global(sym_id)));
        self.params(params, is_variadic);
        self.write_line(format_args!(" {{"));
    }

    fn params(&mut self, params: &[LlvmParam], is_variadic: bool) {
        self.write_str("(");
        for (i, param) in params.iter().enumerate() {
            self.write_fmt(format_args!("{}{param}", if i == 0 { "" } else { ", " }));
        }
        if is_variadic {
            self.write_fmt(format_args!("{}...", if params.is_empty() { "" } else { ", " }));
        }
        self.write_str(")");
    }

    pub fn end_function(&mut self, f: SymbolId) {
        if !self.has_block_ret {
            let sym = f.resolve();
            let &ResolvedType::Function { ret, .. } = sym.ty.id.resolve() else { unreachable!() };
            match ReturnAttr::classify_return(ret) {
                ReturnAttr::Void | ReturnAttr::Sret { .. } => self.ret_void(),
                ReturnAttr::Direct(_) => self.ret(LlvmSymbol::zero(ret)),
            }
        }
        let _ = self.w.write_all(b"}\n");
    }

    pub fn alloca(&mut self, ty: LlvmType) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = alloca {ty}"));
        LlvmSymbol::ptr(r)
    }

    pub fn load(&mut self, ty: LlvmType, slot: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = load {ty}, {slot}"));
        LlvmSymbol::new(ty, r)
    }

    pub fn store(&mut self, src: LlvmSymbol, dst: LlvmSymbol) -> LlvmSymbol {
        self.write_line(format_args!("  store {src}, {dst}"));
        src
    }

    pub fn binop(&mut self, op: &'static str, lhs: LlvmSymbol, rhs: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = {op} {lhs}, {}", rhs.name));
        LlvmSymbol::new(lhs.ty, r)
    }

    pub fn unop(&mut self, op: &'static str, v: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = {op} {v}"));
        LlvmSymbol::new(v.ty, r)
    }

    pub fn cmp(&mut self, op: &'static str, lhs: LlvmSymbol, rhs: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = {op} {lhs}, {}", rhs.name));
        LlvmSymbol::new(LlvmType::bool(), r)
    }

    pub fn convert(&mut self, conv: &'static str, from: LlvmSymbol, to: LlvmType) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = {conv} {from} to {to}"));
        LlvmSymbol::new(to, r)
    }

    pub fn ret(&mut self, b: LlvmSymbol) {
        self.write_line(format_args!("  ret {b}"));
        self.has_block_ret = true;
    }

    pub fn ret_void(&mut self) {
        self.write_line(format_args!("  ret void"));
        self.has_block_ret = true;
    }

    pub fn br_cond(&mut self, cond: LlvmSymbol, l1: LlvmName, l2: LlvmName) {
        self.write_line(format_args!("  br {cond}, label {l1}, label {l2}"));
        self.has_block_ret = true;
    }

    pub fn br(&mut self, l: LlvmName) {
        self.write_line(format_args!("  br label {l}"));
        self.has_block_ret = true;
    }

    pub fn label(&mut self, l: LlvmName) {
        self.has_block_ret = false;
        self.blank();
        self.current_block = l;
        match l {
            LlvmName::Label(i) => self.write_line(format_args!(".l{i}:")),
            LlvmName::NamedLabel(s) => self.write_line(format_args!(".ln.{}:", s.resolve())),
            LlvmName::BreakLabel(i) => self.write_line(format_args!(".brk.{}:", usize::from(i))),
            LlvmName::ContinueLabel(i) => self.write_line(format_args!(".cnt.{}:", usize::from(i))),
            LlvmName::CaseLabel(i) => self.write_line(format_args!(".case.{}:", usize::from(i))),
            _ => unimplemented!(),
        }
    }

    pub fn phi(&mut self, s1: LlvmSymbol, l1: LlvmName, s2: LlvmSymbol, l2: LlvmName) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = phi {} [ {}, {l1} ], [ {}, {l2} ]", s1.ty, s1.name, s2.name));
        LlvmSymbol::new(s1.ty, r)
    }

    pub fn call(&mut self, fty: LlvmType, ret_attr: ReturnAttr, f: LlvmSymbol, params: &[LlvmParam]) -> LlvmSymbol {
        match ret_attr {
            ReturnAttr::Void | ReturnAttr::Sret { .. } => self.call_void(fty, f.name, params),
            ReturnAttr::Direct(ty) => self.call_ret(fty, ty, f.name, params),
        }
    }

    fn call_void(&mut self, fty: LlvmType, f_name: LlvmName, params: &[LlvmParam]) -> LlvmSymbol {
        self.write_fmt(format_args!("  call {fty} {f_name}"));
        self.params(params, false);
        self.blank();
        LlvmSymbol::void()
    }

    fn call_ret(&mut self, fty: LlvmType, ret: LlvmType, f_name: LlvmName, params: &[LlvmParam]) -> LlvmSymbol {
        let r = self.fresh();
        self.write_fmt(format_args!("  {r} = call {fty} {f_name}"));
        self.params(params, false);
        self.blank();
        LlvmSymbol::new(ret, r)
    }

    pub fn gep(&mut self, elem: LlvmType, base: LlvmSymbol, idx: LlvmSymbol) -> LlvmSymbol {
        let r = self.fresh();
        self.write_line(format_args!("  {r} = getelementptr inbounds {elem}, ptr {}, {idx}", base.name));
        LlvmSymbol::ptr(r)
    }

    pub fn aggregate_constant(&mut self, name: LlvmName, qty: QualifiedType, init: &Initializer) {
        let init = LlvmInit::new(qty, Some(init));
        self.write_line(format_args!("{} = private unnamed_addr constant {}", name, init));
        self.blank();
    }

    pub fn string_literal(&mut self, name: LlvmName, str: &StringConstant) {
        let len = str.units.len() + 1;
        let ty = if str.is_wide { LlvmType::int() } else { LlvmType::char() };
        self.write_fmt(format_args!("{name} = private unnamed_addr constant "));
        self.array(ty, str.units.iter().chain(once(&0)), len);
        self.blank();
    }

    fn array<T, I>(&mut self, ty: LlvmType, elems: I, len: usize)
    where
        I: IntoIterator<Item = T>,
        T: Display,
    {
        self.write_fmt(format_args!("[{} x {ty}] [", len));
        for (i, c) in elems.into_iter().enumerate() {
            self.write_fmt(format_args!("{ty} {c}"));
            if i != len - 1 {
                self.write_str(", ");
            }
        }
        self.write_str("]");
    }

    pub fn define_type(&mut self, id: TagDefId) {
        let def = id.resolve();
        let ty = LlvmType::Tag(id);
        match def.kind {
            Tag::Enum => (),
            _ if !def.is_complete => self.write_line(format_args!("{ty} = type opaque")),
            Tag::Struct => self.struct_def(ty, def),
            Tag::Union => self.union_def(ty, def),
        }
    }

    fn struct_def(&mut self, ty: LlvmType, def: &TagDef) {
        self.write_fmt(format_args!("{ty} = type <{{ "));
        for (i, element) in struct_elements(def, ty.size()).iter().enumerate() {
            let sep = if i == 0 { "" } else { ", " };
            self.write_fmt(format_args!("{sep}{}", element.ty()));
        }
        self.write_line(format_args!(" }}>"));
    }

    fn union_def(&mut self, ty: LlvmType, def: &TagDef) {
        let size = ty.size();
        let members = def.members.iter().filter_map(|member| member.symbol).map(|sym| sym.resolve().ty);
        let Some(widest) = members.max_by_key(|qty| sema().layout(&qty.id).align) else {
            return self.write_line(format_args!("{ty} = type {{ [{size} x {}] }}", LlvmType::char()));
        };
        match size - sema().layout(&widest.id).size {
            0 => self.write_line(format_args!("{ty} = type {{ {} }}", widest.llvm())),
            pad => self.write_line(format_args!("{ty} = type {{ {}, [{pad} x {}] }}", widest.llvm(), LlvmType::char())),
        }
    }

    pub fn declare_function(&mut self, sym_id: SymbolId) {
        let sym = sym_id.resolve();
        let ResolvedType::Function { ret, params } = sym.ty.id.resolve() else {
            unreachable!("declare on a non-function")
        };
        let ret_attr = ReturnAttr::classify_return(*ret);
        let args = ret_attr
            .params(
                params,
                |&ty, &align| LlvmParam::unnamed(LlvmType::Ptr, ParamAttr::SRet { ty, align }),
                |p| {
                    let (ty, attr) = classify_param(*p);
                    LlvmParam::unnamed(ty, attr)
                },
            )
            .collect::<Vec<_>>();

        self.write_fmt(format_args!("declare {} {}", ret_attr.ret_llvm(), LlvmName::Global(sym_id)));
        self.params(args.as_slice(), params.is_variadic());
        self.blank();
    }

    pub fn define_global(&mut self, sym_id: SymbolId) {
        let sym = sym_id.resolve();
        let name = LlvmName::Global(sym_id);
        let align = sema().layout(&sym.ty.id).align;
        let kind = if sym.ty.is_const { "constant" } else { "global" };
        if sym.definition == DefinitionState::Declared {
            return self.write_line(format_args!("{name} = external {kind} {}, align {align}", sym.ty.llvm()));
        }
        let linkage = match sym.linkage {
            Linkage::External => "",
            Linkage::Internal | Linkage::None => "internal ",
        };
        let init = LlvmInit::new(sym.ty, sym.initializer.map(|i| i.resolve()));
        self.write_line(format_args!("{name} = {linkage}{kind} {init}, align {align}"));
    }

    pub fn switch(&mut self, condition: LlvmSymbol, cases: &[(LlvmSymbol, LlvmName)], default_l: LlvmName) {
        self.write_line(format_args!("  switch {}, label {} [", condition, default_l));
        for (value, label) in cases {
            self.write_line(format_args!("    {}, label {}", value, label));
        }
        self.write_line(format_args!("  ]"));
        self.has_block_ret = true;
    }

    pub fn memcpy(&mut self, dst: LlvmName, src: LlvmName, layout: Layout) {
        let Layout { size, align } = layout;
        let ptr_len = LlvmType::integer(sema().target.pointer.size);
        self.write_line(format_args!(
            "  call void @llvm.memcpy.p0.p0.{ptr_len}(ptr align {align} {dst}, ptr align {align} {src}, {ptr_len} {size}, i1 false)"
        ));
    }
}
