use std::io::Cursor;
use std::process::{Command, Stdio};
use std::io::Write;

use cc1::ast::{Expression, Name, Tag, Value};
use cc1::context::Context;
use cc1::parser::parse_reader;
use cc1::semantic::{
    Analyzer, Diagnosis, DiagnosisNode, ExpressionKind, ParamTypes, QualifiedType, ResolvedType, SymbolKind,
};

use crate::common::ty::Ty;

fn needs_preprocessing(src: &str) -> bool {
    src.contains("\\\n") || src.contains("/*") || src.contains("//") || src.contains('#')
}

fn preprocess(src: &str) -> String {
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

fn new_ctx() -> Context {
    let mut ctx = Context::default();
    ctx.set_file_name("<test>".to_string());
    ctx
}

impl Unit {
    pub fn parse(src: &str) -> Self {
        let src = preprocess(src);
        let (ctx, status) = parse_reader(new_ctx(), Cursor::new(src));
        Self { ctx, status }
    }

    pub fn compile(src: &str) -> Self {
        let src = preprocess(src);
        let (ctx, status) = parse_reader(new_ctx(), Cursor::new(src));
        let ctx = Analyzer::analyze(ctx);
        Self { ctx, status }
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

    pub fn expressions(&self) -> Vec<String> {
        self.ctx
            .arenas
            .expressions
            .iter()
            .map(|expression| expression.to_string())
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
        self.const_values().into_iter().map(repr).collect()
    }

    pub fn shapes(&self) -> Vec<crate::common::ty::Shape> {
        use crate::common::ty::Shape;

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
                    result_cast: resolved.result_cast.map(|cast| (cast.kind, self.ty_tree(cast.to))),
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
                let name = match self.ctx.arenas.expressions.iter().nth(id).unwrap() {
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
            .iter()
            .map(|symbol| {
                (
                    symbol.name.id.resolve(&self.ctx).clone(),
                    symbol.kind.to_string(),
                    symbol
                        .ty
                        .map(|ty| ty.describe(&self.ctx.sema, &self.ctx).to_string())
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

    pub fn uses(&self) -> Vec<(String, bool)> {
        self.ctx
            .sema
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Function))
            .map(|symbol| (symbol.name.id.resolve(&self.ctx).clone(), symbol.used))
            .collect()
    }

    pub fn tag_members(&self, tag: &str) -> Vec<(String, u32, u32)> {
        let def = self
            .ctx
            .sema
            .tags
            .iter()
            .find(|def| def.name.is_some_and(|name| name.id.resolve(&self.ctx).as_str() == tag))
            .unwrap_or_else(|| panic!("no tag `{tag}` in the unit"));
        def.members
            .iter()
            .filter_map(|m| {
                let sym = m.sym?;
                let name = self.ctx.sema.symbols.get(sym).name.id.resolve(&self.ctx).clone();
                Some((name, m.offset, m.bit_offset))
            })
            .collect()
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

/// Renders a folded/cast constant the same way for `test_target`, `test_value` and
/// `Unit::folded` — `None` for a value the target could not represent.
pub fn repr(value: Option<Value>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "None".to_string(),
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

/// Compiles `src` and asserts it parses and is accepted, returning the resulting `Unit`.
pub fn accepted(src: &str) -> Unit {
    let unit = Unit::compile(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    unit
}

/// Compiles `src`, asserts it parses, and returns its folded constant expressions.
pub fn folded(src: &str) -> Vec<String> {
    let unit = Unit::compile(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    unit.folded()
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

pub fn run_offsets(name: &str, decl: &str, ty: &str, tag: &str, expected: &[(&str, u32, u32)]) {
    let src = format!("{decl} enum layout_probe {{ PROBE = sizeof({ty}) }};");
    let unit = Unit::compile(&src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );

    let got = unit.tag_members(tag);
    let expected: Vec<(String, u32, u32)> = expected.iter().map(|(n, o, b)| (n.to_string(), *o, *b)).collect();
    assert_eq!(got, expected, "`{name}` member offsets in `{tag}`:\n{decl}");
}

pub fn run_uses(name: &str, src: &str, expected: &[(&str, bool)]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );

    let got = unit.uses();
    let expected: Vec<(String, bool)> = expected.iter().map(|(n, u)| (n.to_string(), *u)).collect();
    assert_eq!(got, expected, "`{name}` symbol uses:\n{src}");
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
