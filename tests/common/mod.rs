#![allow(dead_code)]

use std::io::{Cursor, Write};
use std::process::{Command, Stdio};

use cc1::ast::{Expression, Name};
use cc1::parser::{Context, YYLex, Yacc};
use cc1::semantic::{Analyzer, Diagnosis, DiagnosisNode, SymbolKind};

pub const GCC_FLAGS: &[&str] = &[
    "-m32",
    "-fsyntax-only",
    "-std=iso9899:1990",
    "-pedantic-errors",
    "-Wno-deprecated-non-prototype",
    "-Wno-strict-prototypes",
    "-fno-asm",
    "-fno-builtin",
];

fn pipe(program: &str, args: &[&str], input: &str) -> (bool, String) {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("run {program} ({e}) — is it installed?"));

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write source");

    let out = child.wait_with_output().expect("wait");
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn gcc_accepts(src: &str) -> bool {
    let mut args = GCC_FLAGS.to_vec();
    args.extend_from_slice(&["-xc", "-"]);
    pipe("gcc", &args, &format!("{src}\n")).0
}

pub fn preprocess(src: &str) -> String {
    let (ok, out) = pipe("clang", &["-E", "-std=c89", "-xc", "-"], &format!("{src}\n"));
    assert!(ok, "clang -E failed on:\n{src}");
    out
}

pub struct Unit {
    pub ctx: Context,
    pub status: i32,
}

impl Unit {
    pub fn parse(src: &str) -> Self {
        let ctx = Context::new("<test>".to_string());
        let lexer = YYLex::new(Cursor::new(preprocess(src)), || None, ctx);
        let mut yacc = Yacc::new(lexer);
        let status = yacc.yyparse();
        Self {
            ctx: yacc.lexer.ctx,
            status,
        }
    }

    pub fn compile(src: &str) -> Self {
        let mut unit = Self::parse(src);
        if unit.parsed() {
            unit.ctx = Analyzer::analyze(unit.ctx);
        }
        unit
    }

    pub fn parsed(&self) -> bool {
        self.status == 0
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

pub fn run_syntax(name: &str, src: &str) {
    let unit = Unit::parse(src);

    assert!(gcc_accepts(src), "`{name}` invalid — gcc rejected:\n{src}");
    assert!(unit.parsed(), "cc1 failed to parse `{name}`:\n{src}");
}

pub fn run_case(name: &str, src: &str) {
    let unit = Unit::compile(src);
    let gcc = gcc_accepts(src);

    assert_eq!(
        unit.accepts(),
        gcc,
        "failed `{name}`: gcc {} but cc1 {}\nsource:\n{src}\n{}",
        if gcc { "accepts" } else { "rejects" },
        if unit.accepts() { "accepts" } else { "rejects" },
        unit.render(),
    );
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

pub fn run_literal(name: &str, src: &str, expected: &str) {
    let unit = Unit::parse(src);

    assert!(gcc_accepts(src), "`{name}` invalid — gcc rejected:\n{src}");
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
macro_rules! case {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_case(stringify!($name), $src);
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

            assert!(
                !$crate::common::gcc_accepts($src),
                "`{name}` should be rejected by gcc:\n{}",
                $src
            );

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
