#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};

use cc1::ast::print::AstPrinter;
use cc1::ast::{Expression, Name, Tag, Value};
use cc1::context::Context;
use cc1::parser::{YYLex, Yacc};
use cc1::pipeline::Pipeline;
use cc1::semantic::{
    Analyzer, CastKind, Diagnosis, DiagnosisNode, ExpressionKind, ParamTypes, QualifiedType, ResolvedType, SymbolKind,
};

fn needs_preprocessing(src: &str) -> bool {
    src.contains("\\\n") || src.contains("/*") || src.contains("//") || src.contains('#')
}

pub fn preprocess(src: &str) -> String {
    if !needs_preprocessing(src) {
        return format!("{src}\n");
    }

    let mut child = Command::new("clang")
        .args(["-E", "-std=c89", "-xc", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("run clang ({e}) — is it installed?"));

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(format!("{src}\n").as_bytes())
        .expect("write source");

    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "clang -E failed on:\n{src}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub struct Unit {
    pub ctx: Context,
    pub status: i32,
}

thread_local! {
    static SOURCE: RefCell<String> = const { RefCell::new(String::new()) };
    static STATUS: Cell<i32> = const { Cell::new(0) };
}

fn name(mut ctx: Context) -> Context {
    ctx.set_file_name("<test>".to_string());
    ctx
}

fn parse(ctx: Context) -> Context {
    let src = SOURCE.with(|source| source.borrow().clone());
    let lexer = YYLex::new(Cursor::new(src), || None, ctx);
    let mut yacc = Yacc::new(lexer);
    STATUS.set(yacc.yyparse());
    yacc.lexer.ctx
}

impl Unit {
    pub fn parse(src: &str) -> Self {
        SOURCE.set(preprocess(src));
        let (ctx, _) = Pipeline::default().pass(name).pass(parse).finish();
        Self {
            ctx,
            status: STATUS.get(),
        }
    }

    pub fn compile(src: &str) -> Self {
        SOURCE.set(preprocess(src));
        let (ctx, _) = Pipeline::default()
            .pass(name)
            .pass(parse)
            .pass(Analyzer::analyze)
            .finish();
        Self {
            ctx,
            status: STATUS.get(),
        }
    }

    pub fn parsed(&self) -> bool {
        self.status == 0
            && !self
                .diagnosis()
                .iter()
                .any(|diag| matches!(diag.inner, Diagnosis::SyntaxError { .. }))
    }

    pub fn diagnosis(&self) -> &[DiagnosisNode] {
        &self.ctx.diagnosis
    }

    pub fn accepts(&self) -> bool {
        self.parsed() && self.ctx.diagnosis.is_empty()
    }

    pub fn variants(&self) -> Vec<(String, String)> {
        self.ctx
            .sema
            .symbols
            .data
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::Variant)
            .map(|symbol| {
                (
                    symbol.name.id.resolve(&self.ctx).clone(),
                    symbol.value.map(|v| v.to_string()).unwrap_or_default(),
                )
            })
            .collect()
    }

    pub fn string_literals(&self) -> Vec<String> {
        self.ctx
            .arenas
            .expressions
            .data
            .iter()
            .filter_map(|expression| match expression {
                Expression::StringLiteral(literal) => {
                    let text = literal.name().id.resolve(&self.ctx);
                    Some(match literal.is_wide() {
                        true => format!("L{text}"),
                        false => text.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn const_values(&self) -> Vec<Option<Value>> {
        self.ctx
            .sema
            .expr_facts()
            .iter()
            .filter(|facts| facts.constant_seen)
            .map(|facts| facts.constant)
            .collect()
    }

    pub fn folded(&self) -> Vec<String> {
        self.const_values()
            .into_iter()
            .map(|value| match value {
                Some(value) => format!("{value:?}"),
                None => "None".to_string(),
            })
            .collect()
    }

    pub fn shapes(&self) -> Vec<Shape> {
        self.ctx
            .sema
            .expr_facts()
            .iter()
            .filter(|facts| facts.resolved_seen)
            .map(|facts| match &facts.resolved {
                Some(resolved) => Shape {
                    ty: Some(self.ty_tree(resolved.ty)),
                    lvalue: matches!(resolved.kind, ExpressionKind::LValue),
                    casts: resolved
                        .casts
                        .iter()
                        .map(|cast| (cast.kind, self.ty_tree(cast.to)))
                        .collect(),
                },
                None => Shape::unresolved(),
            })
            .collect()
    }

    pub fn ty_tree(&self, qt: QualifiedType) -> Ty {
        let base = match qt.id.resolve(&self.ctx) {
            ResolvedType::Void => Ty::Void,
            ResolvedType::Char => Ty::Char,
            ResolvedType::SignedChar => Ty::SChar,
            ResolvedType::UnsignedChar => Ty::UChar,
            ResolvedType::Short => Ty::Short,
            ResolvedType::UnsignedShort => Ty::UShort,
            ResolvedType::Int => Ty::Int,
            ResolvedType::UnsignedInt => Ty::UInt,
            ResolvedType::Long => Ty::Long,
            ResolvedType::UnsignedLong => Ty::ULong,
            ResolvedType::Float => Ty::Float,
            ResolvedType::Double => Ty::Double,
            ResolvedType::LongDouble => Ty::LDouble,
            ResolvedType::Pointer(inner) => Ty::Ptr(Box::new(self.ty_tree(*inner))),
            ResolvedType::Array { elem, len } => Ty::Array(Box::new(self.ty_tree(*elem)), *len),
            ResolvedType::Function { ret, params } => {
                let variadic = matches!(params, ParamTypes::Prototype { is_variadic: true, .. });
                let params = match params {
                    ParamTypes::Unspecified => None,
                    ParamTypes::Prototype { params, .. } => Some(params.iter().map(|p| self.ty_tree(*p)).collect()),
                };
                Ty::Func {
                    ret: Box::new(self.ty_tree(*ret)),
                    params,
                    variadic,
                }
            }
            ResolvedType::Tag(tag) => {
                let def = tag.resolve(&self.ctx);
                let name = def.name.map(|n| n.id.resolve(&self.ctx).clone());
                let complete = def.is_complete;
                match def.kind {
                    Tag::Struct => Ty::Struct { tag: name, complete },
                    Tag::Union => Ty::Union { tag: name, complete },
                    Tag::Enum => Ty::Enum { tag: name, complete },
                }
            }
        };
        let base = if qt.is_volatile { Ty::Volatile(Box::new(base)) } else { base };
        if qt.is_const { Ty::Const(Box::new(base)) } else { base }
    }

    pub fn symbol_ty_tree(&self, name: &str) -> Ty {
        self.ty_tree(self.symbol_ty(name))
    }

    pub fn symbol_ty(&self, name: &str) -> QualifiedType {
        self.ctx
            .sema
            .symbols
            .data
            .iter()
            .find(|symbol| symbol.name.id.resolve(&self.ctx).as_str() == name)
            .unwrap_or_else(|| panic!("no symbol `{name}` in the unit"))
            .ty
            .unwrap_or_else(|| panic!("symbol `{name}` has no type"))
    }

    pub fn prim(&self, name: &str) -> QualifiedType {
        let b = &self.ctx.sema.builtins;
        let id = match name {
            "void" => b.void,
            "char" => b.char,
            "signed char" => b.signed_char,
            "unsigned char" => b.unsigned_char,
            "short" => b.short,
            "unsigned short" => b.unsigned_short,
            "int" => b.int,
            "unsigned" | "unsigned int" => b.unsigned_int,
            "long" => b.long,
            "unsigned long" => b.unsigned_long,
            "float" => b.float,
            "double" => b.double,
            "long double" => b.long_double,
            other => panic!("unknown primitive `{other}`"),
        };
        QualifiedType::new(id, false, false)
    }

    pub fn bindings(&self) -> Vec<(String, Option<usize>)> {
        self.ctx
            .sema
            .expr_facts()
            .iter()
            .enumerate()
            .filter(|(_, facts)| facts.binding_seen)
            .map(|(id, facts)| {
                let name = match &self.ctx.arenas.expressions.data[id] {
                    Expression::Identifier(name) => name.id.resolve(&self.ctx).clone(),
                    other => format!("{other:?}"),
                };
                (name, facts.binding.map(usize::from))
            })
            .collect()
    }

    pub fn symbols(&self) -> Vec<(String, String, String)> {
        self.ctx
            .sema
            .symbols
            .data
            .iter()
            .map(|symbol| {
                (
                    symbol.name.id.resolve(&self.ctx).clone(),
                    symbol.kind.to_string(),
                    symbol
                        .ty
                        .map(|ty| ty.describe(&self.ctx.sema, &self.ctx))
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    pub fn describe(&self, name: &str) -> Option<String> {
        self.symbols()
            .into_iter()
            .find(|(symbol, _, _)| symbol == name)
            .map(|(_, _, ty)| ty)
    }

    pub fn messages(&self) -> Vec<String> {
        self.diagnosis()
            .iter()
            .map(|diag| {
                let mut buf = Vec::new();
                let _ = diag.write(&mut buf, &self.ctx);
                strip_ansi(&String::from_utf8_lossy(&buf)).trim_end().to_string()
            })
            .collect()
    }

    pub fn ast(&self) -> String {
        let mut buf = Vec::new();
        let _ = AstPrinter::write_ast(&mut buf, &self.ctx);
        strip_ansi(&String::from_utf8_lossy(&buf))
            .lines()
            .map(|line| line.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for diag in self.diagnosis() {
            let mut buf = Vec::new();
            let _ = diag.write(&mut buf, &self.ctx);
            out.push_str(&String::from_utf8_lossy(&buf));
        }
        out
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ty {
    Void,
    Char,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    LDouble,
    Ptr(Box<Ty>),
    Array(Box<Ty>, Option<usize>),
    Func {
        ret: Box<Ty>,
        params: Option<Vec<Ty>>,
        variadic: bool,
    },
    Struct {
        tag: Option<String>,
        complete: bool,
    },
    Union {
        tag: Option<String>,
        complete: bool,
    },
    Enum {
        tag: Option<String>,
        complete: bool,
    },
    Const(Box<Ty>),
    Volatile(Box<Ty>),
}

impl Ty {
    pub fn ptr(inner: Ty) -> Ty {
        Ty::Ptr(Box::new(inner))
    }

    pub fn arr(elem: Ty, len: usize) -> Ty {
        Ty::Array(Box::new(elem), Some(len))
    }

    pub fn flex(elem: Ty) -> Ty {
        Ty::Array(Box::new(elem), None)
    }

    pub fn func(ret: Ty, params: impl IntoIterator<Item = Ty>) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(params.into_iter().collect()),
            variadic: false,
        }
    }

    pub fn func0(ret: Ty) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(Vec::new()),
            variadic: false,
        }
    }

    pub fn func_variadic(ret: Ty, params: impl IntoIterator<Item = Ty>) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(params.into_iter().collect()),
            variadic: true,
        }
    }

    pub fn noproto(ret: Ty) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: None,
            variadic: false,
        }
    }

    pub fn strukt(tag: &str) -> Ty {
        Ty::Struct {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn strukt_incomplete(tag: &str) -> Ty {
        Ty::Struct {
            tag: Some(tag.to_string()),
            complete: false,
        }
    }

    pub fn anon_struct() -> Ty {
        Ty::Struct {
            tag: None,
            complete: true,
        }
    }

    pub fn union(tag: &str) -> Ty {
        Ty::Union {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn enom(tag: &str) -> Ty {
        Ty::Enum {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn konst(inner: Ty) -> Ty {
        Ty::Const(Box::new(inner))
    }

    pub fn vol(inner: Ty) -> Ty {
        Ty::Volatile(Box::new(inner))
    }
}

#[derive(Debug, PartialEq)]
pub struct Shape {
    pub ty: Option<Ty>,
    pub lvalue: bool,
    pub casts: Vec<(CastKind, Ty)>,
}

impl Shape {
    pub fn rvalue(ty: Ty) -> Self {
        Self {
            ty: Some(ty),
            lvalue: false,
            casts: Vec::new(),
        }
    }

    pub fn lvalue(ty: Ty) -> Self {
        Self {
            ty: Some(ty),
            lvalue: true,
            casts: Vec::new(),
        }
    }

    pub fn unresolved() -> Self {
        Self {
            ty: None,
            lvalue: false,
            casts: Vec::new(),
        }
    }

    pub fn then(mut self, kind: CastKind, to: Ty) -> Self {
        self.casts.push((kind, to));
        self
    }
}

pub fn rv(ty: Ty) -> Shape {
    Shape::rvalue(ty)
}

pub fn lv(ty: Ty) -> Shape {
    Shape::lvalue(ty)
}

pub fn none() -> Shape {
    Shape::unresolved()
}

pub fn ints(n: usize) -> Vec<Shape> {
    (0..n).map(|_| rv(Ty::Int)).collect()
}

pub fn strip_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        for c in chars.by_ref() {
            if c == 'm' {
                break;
            }
        }
    }
    out
}

pub fn run_syntax(name: &str, src: &str) {
    let unit = Unit::parse(src);

    assert!(unit.parsed(), "cc1 failed to parse `{name}`:\n{src}");
}

pub fn run_accept(name: &str, src: &str) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` should parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` should be accepted:\n{src}\n{}",
        unit.render()
    );
}

pub fn run_reject(name: &str, src: &str) {
    let unit = Unit::compile(src);

    assert!(!unit.accepts(), "`{name}` should be rejected:\n{src}");
}

pub fn run_value(name: &str, src: &str, expected: &[(&str, &str)]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );

    let variants = unit.variants();
    for (variant, value) in expected {
        let found = variants.iter().find(|(name, _)| name == variant);
        assert_eq!(
            found.map(|(_, value)| value.as_str()),
            Some(*value),
            "`{name}` variant `{variant}` should be {value}:\n{src}\n{variants:?}"
        );
    }
}

pub fn run_size(name: &str, decl: &str, ty: &str, expected: u64) {
    let src = format!("{decl} enum layout_probe {{ PROBE = sizeof({ty}) }};");
    let unit = Unit::compile(&src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );

    let variants = unit.variants();
    let got = variants.iter().find(|(name, _)| name == "PROBE");
    assert_eq!(
        got.map(|(_, value)| value.as_str()),
        Some(expected.to_string().as_str()),
        "`{name}` sizeof({ty}) should be {expected}:\n{decl}"
    );
}

pub fn run_literal(name: &str, src: &str, expected: &str) {
    let unit = Unit::parse(src);

    assert!(unit.parsed(), "cc1 failed to parse `{name}`:\n{src}");
    assert_eq!(
        unit.string_literals(),
        vec![expected.to_string()],
        "wrong string literal value for `{name}`:\n{src}"
    );
}

fn diagnosis_name(diagnosis: &Diagnosis) -> Option<Name> {
    match diagnosis {
        Diagnosis::UndeclaredIdentifier(name) => Some(*name),
        Diagnosis::DuplicateDeclaration(_, name) => Some(*name),
        _ => None,
    }
}

pub fn assert_unmentioned(name: &str, src: &str, unit: &Unit, forbidden: &[&str]) {
    for word in forbidden {
        for diag in unit.diagnosis() {
            let mentioned = diagnosis_name(&diag.inner).map(|n| n.id.resolve(&unit.ctx).as_str() == *word);
            assert!(
                mentioned != Some(true),
                "`{name}` cascading diagnosis mentions {word}:\n{src}\n{}",
                unit.render()
            );
        }
    }
}

#[macro_export]
macro_rules! accept {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_accept(stringify!($name), $src);
        }
    };
}

#[macro_export]
macro_rules! reject {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_reject(stringify!($name), $src);
        }
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_syntax(stringify!($name), $src);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            $crate::common::run_syntax(stringify!($name), $src);
        }
    };
}

#[macro_export]
macro_rules! value {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_value(stringify!($name), $src, $expected);
        }
    };
}

#[macro_export]
macro_rules! literal {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_literal(stringify!($name), $src, $expected);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            $crate::common::run_literal(stringify!($name), $src, $expected);
        }
    };
}

#[macro_export]
macro_rules! recover {
    ($name:ident, $src:expr, [$($diag:pat),* $(,)?], $forbidden:expr) => {
        #[test]
        fn $name() {
            let name = stringify!($name);
            let unit = $crate::common::Unit::compile($src);

            let mut got = unit.diagnosis().iter();
            $(
                let next = got.next().map(|diag| diag.inner.clone());
                assert!(
                    matches!(next, Some($diag)),
                    "`{name}` expected {}, got {next:?}:\n{}\n{}",
                    stringify!($diag),
                    $src,
                    unit.render()
                );
            )*
            let extra: Vec<_> = got.map(|diag| diag.inner.clone()).collect();
            assert!(
                extra.is_empty(),
                "`{name}` unexpected extra diagnosis {extra:?}:\n{}\n{}",
                $src,
                unit.render()
            );

            $crate::common::assert_unmentioned(name, $src, &unit, $forbidden);
        }
    };
}

#[macro_export]
macro_rules! size {
    ($name:ident, $decl:expr, $ty:expr, $expected:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_size(stringify!($name), $decl, $ty, $expected);
        }
    };
}
