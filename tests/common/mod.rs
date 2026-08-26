#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};

use cc1::ast::print::AstPrinter;
use cc1::ast::{Expression, Name, Value};
use cc1::parser::{Context, YYLex, Yacc};
use cc1::pipeline::Pipeline;
use cc1::semantic::{Analyzer, Diagnosis, DiagnosisNode, SymbolKind};

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
            .arenas
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
                Expression::StringLiteral(name) => Some(name.id.resolve(&self.ctx).clone()),
                _ => None,
            })
            .collect()
    }

    pub fn const_values(&self) -> Vec<Option<Value>> {
        let mut entries: Vec<_> = self
            .ctx
            .const_values
            .iter()
            .map(|(id, value)| (usize::from(*id), *value))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        entries.into_iter().map(|(_, value)| value).collect()
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

    pub fn bindings(&self) -> Vec<(String, Option<usize>)> {
        let mut entries: Vec<_> = self
            .ctx
            .bindings
            .iter()
            .map(|(id, symbol)| (usize::from(*id), symbol.map(usize::from)))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        entries
            .into_iter()
            .map(|(id, symbol)| {
                let name = match &self.ctx.arenas.expressions.data[id] {
                    Expression::Identifier(name) => name.id.resolve(&self.ctx).clone(),
                    other => format!("{other:?}"),
                };
                (name, symbol)
            })
            .collect()
    }

    pub fn symbols(&self) -> Vec<(String, String, String)> {
        self.ctx
            .arenas
            .symbols
            .data
            .iter()
            .map(|symbol| {
                (
                    symbol.name.id.resolve(&self.ctx).clone(),
                    symbol.kind.to_string(),
                    symbol.ty.map(|ty| self.ctx.describe(&ty)).unwrap_or_default(),
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
                let next = got.next().map(|diag| diag.inner);
                assert!(
                    matches!(next, Some($diag)),
                    "`{name}` expected {}, got {next:?}:\n{}\n{}",
                    stringify!($diag),
                    $src,
                    unit.render()
                );
            )*
            let extra: Vec<_> = got.map(|diag| diag.inner).collect();
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
