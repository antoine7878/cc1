use std::io::Cursor;
use std::io::Write;
use std::process::{Command, Stdio};

use cc1::ast::statement::StatementId;
use cc1::ast::{ConstValue, Expression, ExpressionId, Name, StringConstant, Tag};
use cc1::context::{self, Context, install};
use cc1::parser::parse_reader;
use cc1::semantic::{
    AddressBase, Analyzer, Diagnosis, DiagnosisNode, ExpressionKind, Initializer, ParamTypes, QualifiedType,
    ResolvedStatement, ResolvedType, Sema, SymbolKind, install as install_sema,
};
use cc1::target::{I386, Target};

use crate::common::ty::Ty;

fn string_body(constant: &StringConstant) -> String {
    let mut out = String::new();
    for &unit in &constant.units {
        match char::from_u32(unit) {
            Some(c) if (' '..='~').contains(&c) && c != '\\' && c != '"' => out.push(c),
            _ => out.push_str(&format!("\\x{unit:02x}")),
        }
    }
    out
}

fn string_display(constant: &StringConstant) -> String {
    let prefix = if constant.is_wide { "L" } else { "" };
    format!("{prefix}{}", string_body(constant))
}

fn string_quoted(constant: &StringConstant) -> String {
    let prefix = if constant.is_wide { "L" } else { "" };
    format!("{prefix}\"{}\"", string_body(constant))
}

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
    pub ctx: &'static Context,
    pub sema: &'static Sema,
    pub status: i32,
}

fn ctx_for(target: Target) -> Context {
    let mut ctx = Context::with_target(target);
    ctx.set_file_name("<test>".to_string());
    ctx
}

fn new_ctx() -> Context {
    ctx_for(I386)
}

impl Unit {
    pub fn parse(src: &str) -> Self {
        let src = preprocess(src);
        let (ctx, status) = parse_reader(new_ctx(), Cursor::new(src));
        let ctx = install(ctx);
        let sema = install_sema(Sema::new(ctx.target.clone()));
        Self { ctx, sema, status }
    }

    pub fn compile(src: &str) -> Self {
        Self::compile_for(I386, src)
    }

    pub fn compile_for(target: Target, src: &str) -> Self {
        let src = preprocess(src);
        let (ctx, status) = parse_reader(ctx_for(target), Cursor::new(src));
        let sema = Analyzer::analyze(ctx);
        Self {
            ctx: context::ctx(),
            sema,
            status,
        }
    }

    pub fn parsed(&self) -> bool {
        self.status == 0
            && !self
                .diagnosis()
                .iter()
                .any(|diag| matches!(diag.inner, Diagnosis::SyntaxError { .. }))
    }

    pub fn diagnosis(&self) -> Vec<DiagnosisNode> {
        self.ctx.diagnosis.iter().chain(&self.sema.diagnosis).cloned().collect()
    }

    pub fn accepts(&self) -> bool {
        self.parsed() && self.diagnosis().is_empty()
    }

    pub fn variants(&self) -> Vec<(String, String)> {
        self.sema
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::Variant)
            .map(|symbol| {
                (
                    symbol.name.id.resolve().clone(),
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
                Expression::StringLiteral(literal) => Some(string_display(literal.constant())),
                _ => None,
            })
            .collect()
    }

    pub fn string_pool(&self) -> Vec<String> {
        self.ctx.arenas.strings.iter().map(string_quoted).collect()
    }

    pub fn expressions(&self) -> Vec<String> {
        self.ctx
            .arenas
            .expressions
            .iter()
            .map(|expression| expression.to_string())
            .collect()
    }

    pub fn const_values(&self) -> Vec<Option<ConstValue>> {
        (0..self.sema.expr_consts.len())
            .map(ExpressionId::from)
            .filter(|&id| self.sema.expr_consts.seen(id))
            .map(|id| self.sema.expr_consts.get(id).copied())
            .collect()
    }

    pub fn folded(&self) -> Vec<String> {
        self.const_values().into_iter().map(repr).collect()
    }

    pub fn shapes(&self) -> Vec<crate::common::ty::Shape> {
        use crate::common::ty::Shape;

        (0..self.sema.expr_types.len())
            .map(ExpressionId::from)
            .filter(|&id| self.sema.expr_types.seen(id))
            .map(|id| match self.sema.expr_types.get(id) {
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
        let base = match qt.id.resolve() {
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
                let def = tag.resolve();
                let name = def.name.map(|n| n.id.resolve().clone());
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
        self.sema
            .symbols
            .iter()
            .find(|symbol| symbol.name.id.resolve().as_str() == name)
            .unwrap_or_else(|| panic!("no symbol `{name}` in the unit"))
            .ty
            .unwrap_or_else(|| panic!("symbol `{name}` has no type"))
    }

    pub fn prim(&self, name: &str) -> QualifiedType {
        let b = &self.sema.builtins;
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
        (0..self.sema.expr_bindings.len())
            .map(ExpressionId::from)
            .filter(|&id| self.sema.expr_bindings.seen(id))
            .map(|id| {
                let name = match self.ctx.arenas.expressions.iter().nth(id.into()).unwrap() {
                    Expression::Identifier(name) => name.id.resolve().clone(),
                    other => format!("{other:?}"),
                };
                (name, self.sema.expr_bindings.get(id).copied().map(usize::from))
            })
            .collect()
    }

    pub fn member_refs(&self) -> Vec<(String, String, usize)> {
        (0..self.sema.member_refs.len())
            .map(ExpressionId::from)
            .filter(|&id| self.sema.member_refs.seen(id))
            .filter_map(|id| {
                let reference = self.sema.member_refs.get(id).copied()?;
                let sym = reference.member(self.sema).sym?;
                let member = self.sema.symbols.get(sym).name.id.resolve().clone();
                let tag = self.sema.tags.get(reference.tag).name;
                let tag = tag.map_or_else(|| "<anonymous>".to_string(), |n| n.id.resolve().clone());
                Some((member, tag, reference.index))
            })
            .collect()
    }

    pub fn statements(&self) -> Vec<String> {
        (0..self.sema.stmts.len())
            .map(StatementId::from)
            .filter_map(|id| {
                let fact = self.render_statement(self.sema.stmts.get(id)?);
                Some(format!("#{} {fact}", usize::from(id)))
            })
            .collect()
    }

    fn render_statement(&self, stmt: &ResolvedStatement) -> String {
        match stmt {
            ResolvedStatement::Loop(_) => "loop".to_string(),
            ResolvedStatement::Switch {
                control,
                cases,
                default,
            } => {
                let control = control.to_string();
                let cases: Vec<String> = cases
                    .iter()
                    .map(|(value, id)| format!("{}->#{}", repr(Some(*value)), usize::from(*id)))
                    .collect();
                let default = default.map_or_else(|| "none".to_string(), |id| format!("#{}", usize::from(id)));
                format!("switch {control} [{}] default={default}", cases.join(", "))
            }
            ResolvedStatement::Case(value, target) => {
                format!("case {}->#{}", repr(Some(*value)), usize::from(*target))
            }
            ResolvedStatement::Default(target) => format!("default->#{}", usize::from(*target)),
            ResolvedStatement::Break(target) => format!("break->#{}", usize::from(*target)),
            ResolvedStatement::Continue(target) => format!("continue->#{}", usize::from(*target)),
            ResolvedStatement::Goto(name) => format!("goto {}", name.resolve()),
            ResolvedStatement::Label(name) => format!("label {}", name.resolve()),
        }
    }

    pub fn labels(&self) -> Vec<(String, Vec<String>)> {
        self.sema
            .functions
            .iter()
            .map(|def| {
                let name = self.sema.symbols.get(def.sym).name.id.resolve().clone();
                let labels = def.labels.iter().map(|l| l.id.resolve().clone()).collect();
                (name, labels)
            })
            .collect()
    }

    fn render_initializer(&self, init: &Initializer) -> String {
        match init {
            Initializer::Zero => "0".to_string(),
            Initializer::Value(value) => repr(Some(*value)),
            Initializer::String(id) => string_quoted(id.resolve()),
            Initializer::Expr(_) => "expr".to_string(),
            Initializer::List(items) => {
                let rendered: Vec<String> = items.iter().map(|i| self.render_initializer(i)).collect();
                format!("{{{}}}", rendered.join(", "))
            }
            Initializer::Address(at) => {
                let base = match at.base {
                    AddressBase::Symbol(sym) => self.sema.symbols.get(sym).name.id.resolve().clone(),
                    AddressBase::String(id) => string_quoted(id.resolve()),
                    AddressBase::Absolute => "abs".to_string(),
                };
                match at.offset {
                    0 => format!("&{base}"),
                    n if n > 0 => format!("&{base}+{n}"),
                    n => format!("&{base}{n}"),
                }
            }
        }
    }

    pub fn initializers(&self) -> Vec<(String, String)> {
        self.sema
            .symbols
            .iter()
            .filter_map(|symbol| {
                let id = symbol.initializer?;
                let name = symbol.name.id.resolve().clone();
                Some((name, self.render_initializer(self.sema.inits.get(id))))
            })
            .collect()
    }

    pub fn symbols(&self) -> Vec<(String, String, String)> {
        self.sema
            .symbols
            .iter()
            .map(|symbol| {
                (
                    symbol.name.id.resolve().clone(),
                    symbol.kind.to_string(),
                    symbol.ty.map(|ty| ty.to_string()).unwrap_or_default(),
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
        self.sema
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Function))
            .map(|symbol| (symbol.name.id.resolve().clone(), symbol.used))
            .collect()
    }

    pub fn placements(&self) -> Vec<(String, String, String, String)> {
        self.sema
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Function))
            .map(|symbol| {
                (
                    symbol.name.id.resolve().clone(),
                    symbol.linkage.to_string(),
                    symbol.duration.to_string(),
                    symbol.definition.to_string(),
                )
            })
            .collect()
    }

    pub fn tag_members(&self, tag: &str) -> Vec<(String, u32, u32)> {
        let def = self
            .sema
            .tags
            .iter()
            .find(|def| def.name.is_some_and(|name| name.id.resolve().as_str() == tag))
            .unwrap_or_else(|| panic!("no tag `{tag}` in the unit"));
        def.members
            .iter()
            .filter_map(|m| {
                let sym = m.sym?;
                let name = self.sema.symbols.get(sym).name.id.resolve().clone();
                Some((name, m.offset, m.bit_offset))
            })
            .collect()
    }

    /// Absolute bit position of each named member, the quantity the ABI actually fixes.
    pub fn member_bits(&self, tag: &str) -> Vec<(String, u32)> {
        self.tag_members(tag)
            .into_iter()
            .map(|(name, offset, bit)| (name, offset * 8 + bit))
            .collect()
    }

    pub fn messages(&self) -> Vec<String> {
        self.diagnosis()
            .iter()
            .map(|diag| {
                let mut buf = Vec::new();
                let _ = diag.write(&mut buf);
                strip_ansi(&String::from_utf8_lossy(&buf)).trim_end().to_string()
            })
            .collect()
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for diag in self.diagnosis() {
            let mut buf = Vec::new();
            let _ = diag.write(&mut buf);
            out.push_str(&String::from_utf8_lossy(&buf));
        }
        out
    }
}

/// Renders a folded/cast constant the same way for `test_target`, `test_value` and
/// `Unit::folded` — `None` for a value the target could not represent.
pub fn repr(value: Option<ConstValue>) -> String {
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
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
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
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
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
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{decl}"
    );

    let variants = unit.variants();
    let got = variants.iter().find(|(name, _)| name == "PROBE");
    assert_eq!(
        got.map(|(_, value)| value.as_str()),
        Some(expected.to_string().as_str()),
        "`{name}` sizeof({ty}) should be {expected}:\n{decl}"
    );
}

pub fn run_bits(name: &str, decl: &str, tag: &str, expected: &[(&str, u32)]) {
    let unit = Unit::compile(decl);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{decl}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{decl}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{decl}"
    );

    let expected: Vec<(String, u32)> = expected.iter().map(|(n, b)| (n.to_string(), *b)).collect();
    assert_eq!(
        unit.member_bits(tag),
        expected,
        "`{name}` member bit offsets in `{tag}`:\n{decl}"
    );
}

pub fn run_offsets(name: &str, decl: &str, tag: &str, expected: &[(&str, u32, u32)]) {
    let unit = Unit::compile(decl);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{decl}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{decl}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{decl}"
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
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let got = unit.uses();
    let expected: Vec<(String, bool)> = expected.iter().map(|(n, u)| (n.to_string(), *u)).collect();
    assert_eq!(got, expected, "`{name}` symbol uses:\n{src}");
}

pub fn run_initializers(name: &str, src: &str, expected: &[(&str, &str)]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let got = unit.initializers();
    let expected: Vec<(String, String)> = expected.iter().map(|(s, i)| (s.to_string(), i.to_string())).collect();
    assert_eq!(got, expected, "`{name}` initializers:\n{src}");
}

pub fn run_statements(name: &str, src: &str, expected: &[&str]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    assert_eq!(unit.statements(), expected, "wrong statements for `{name}`:\n{src}");
}

pub fn run_labels(name: &str, src: &str, expected: &[(&str, &[&str])]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let expected: Vec<(String, Vec<String>)> = expected
        .iter()
        .map(|(f, labels)| (f.to_string(), labels.iter().map(|l| l.to_string()).collect()))
        .collect();
    assert_eq!(unit.labels(), expected, "wrong labels for `{name}`:\n{src}");
}

pub fn run_member_refs(name: &str, src: &str, expected: &[(&str, &str, usize)]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let got = unit.member_refs();
    let expected: Vec<(String, String, usize)> = expected
        .iter()
        .map(|(m, t, i)| (m.to_string(), t.to_string(), *i))
        .collect();
    assert_eq!(got, expected, "`{name}` member refs:\n{src}");
}

pub fn run_placements(name: &str, src: &str, expected: &[(&str, &str, &str, &str)]) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );

    let got = unit.placements();
    let expected: Vec<(String, String, String, String)> = expected
        .iter()
        .map(|(n, l, d, f)| (n.to_string(), l.to_string(), d.to_string(), f.to_string()))
        .collect();
    assert_eq!(got, expected, "`{name}` symbol placements:\n{src}");
}

pub fn run_pool(name: &str, src: &str, expected: &[&str]) {
    let unit = Unit::parse(src);

    assert!(unit.parsed(), "cc1 failed to parse `{name}`:\n{src}");
    assert_eq!(
        unit.string_pool(),
        expected.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
        "wrong string pool for `{name}`:\n{src}"
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
            let mentioned = diagnosis_name(&diag.inner).map(|n| n.id.resolve().as_str() == *word);
            assert!(
                mentioned != Some(true),
                "`{name}` cascading diagnosis mentions {word}:\n{src}\n{}",
                unit.render()
            );
        }
    }
}
