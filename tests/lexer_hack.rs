use std::fs;
use std::process::Command;

const GCC_FLAGS: &[&str] = &[
    "-fsyntax-only",
    "-std=iso9899:1990",
    "-pedantic-errors",
    "-fno-gnu-keywords",
    "-Wno-deprecated-non-prototype",
    "-fno-asm",
    "-fno-builtin",
];

fn gcc_accepts(path: &str) -> bool {
    let out = Command::new("gcc")
        .args(GCC_FLAGS)
        .arg(path)
        .output()
        .expect("run gcc (is gcc installed?)");
    out.status.success()
}

// cc1 always exits 0 (no error recovery yet), so the verdict is "stderr empty"
fn cc1_accepts(path: &str) -> bool {
    let out = Command::new(env!("CARGO_BIN_EXE_cc1"))
        .arg(path)
        .output()
        .expect("run cc1 binary");
    out.stderr.is_empty()
}

fn run_case(name: &str, src: &str) {
    let dir = std::env::temp_dir().join(format!("cc1-tests-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("{name}.c"));
    fs::write(&path, src).expect("write case");
    let path = path.to_string_lossy().into_owned();

    let gcc = gcc_accepts(&path);
    let cc1 = cc1_accepts(&path);

    fs::remove_file(&path).ok();

    assert_eq!(
        cc1,
        gcc,
        "verdict mismatch for `{name}`: gcc {} but cc1 {}\nsource:\n{src}",
        if gcc { "accepts" } else { "rejects" },
        if cc1 { "accepts" } else { "rejects" },
    );
}

macro_rules! case {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            run_case(stringify!($name), $src);
        }
    };
}

// ---- typedef registration, lookup, scoping ------------------------------

case!(typedef_then_use, "typedef int T; T x;");

case!(typedef_use_in_block, "typedef int T; void f(void) { T x; x = 1; }");

case!(typedef_chain, "typedef int A; typedef A B; B x;");

case!(typedef_of_struct_tag, "struct S { int x; }; typedef struct S S; S v;");

case!(forward_struct_typedef, "typedef struct S *SP; struct S { SP p; };");

case!(cast_to_typedef, "typedef int T; void f(void) { int x; x = (T)1; }");

case!(
    typedef_redef_in_block,
    "typedef int T; void f(void) { typedef T T; T x; x = 1; }"
);

case!(
    knr_definition_with_typedef_param_type,
    "typedef int T; f(a) T a; { return a; }"
);

case!(block_typedef_no_leak, "void f(void) { typedef int T; } int T;");

// gcc rejects (U undeclared at file scope); cc1 must not leak U out of the block
case!(
    nested_block_typedef_no_leak,
    "typedef int T; void f(void) { if (1) { typedef char U; } } U u;"
);

// ---- rejections both compilers must agree on -----------------------------

// C90 6.1.2.1: enumerator conflicts with a typedef of the same scope
case!(enumerator_conflicts_with_typedef, "typedef int T; enum E { T };");

// gcc: old-style parameter declarations in prototyped function definition
case!(knr_param_named_as_typedef, "typedef int a; f(a) int a; { return a; }");

// verdicts match today, but latent: cc1 classifies sizeof(T) as sizeof(type_name)
// where gcc sees sizeof(expression) once T is shadowed
case!(
    sizeof_shadowed_verdict,
    "typedef int T; int main(void) { int T; return sizeof(T); }"
);

// ---- known divergences (shadowing: inner ordinary decl must hide typedef) -

case!(
    local_var_shadows_typedef,
    "typedef int T; int main(void) { int T; T = 1; return T; }"
);

case!(param_shadows_typedef, "typedef int T; T f(T T) { return T; }");

case!(
    paren_expr_of_shadowed_typedef,
    "typedef int T; int main(void) { int T; return (T); }"
);

case!(
    sizeof_expr_on_shadowed_typedef,
    "typedef int T; int main(void) { int T; return sizeof T; }"
);

case!(label_named_as_typedef, "typedef int T; void f(void) { T: ; goto T; }");

// ---- known divergences (separate namespaces, C90 6.1.2.3) -----------------

// A1: tags live in their own namespace; enum was patched (ENUM TYPE_NAME), struct/union not
case!(struct_tag_named_as_typedef, "typedef int S; struct S { int x; };");

case!(union_tag_named_as_typedef, "typedef int S; union S { int x; };");

// members live in their own namespace
case!(member_named_as_typedef, "typedef int T; struct S { T T; };");

case!(
    member_access_named_as_typedef,
    "typedef int x; struct S { int x; }; void f(void) { struct S s; s.x = 1; }"
);

// ---- known divergences (other) -------------------------------------------

// gcc rejects the redeclaration; cc1 parses it as two type specifiers, no declarator
case!(typedef_redeclared_as_var, "typedef int T; int T;");

// legal C90: struct definition inside a K&R declaration_list
case!(
    knr_struct_def_in_declaration_list,
    "f(a) struct S { int x; } *a; { return 0; }"
);
