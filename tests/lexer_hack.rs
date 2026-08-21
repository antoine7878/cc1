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
    fs::write(&path, format!("{src}\n")).expect("write case");
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

case!(
    nested_block_typedef_no_leak,
    "typedef int T; void f(void) { if (1) { typedef char U; } } U u;"
);

case!(enumerator_conflicts_with_typedef, "typedef int T; enum E { T };");

case!(knr_param_named_as_typedef, "typedef int a; f(a) int a; { return a; }");

case!(
    sizeof_shadowed_verdict,
    "typedef int T; int main(void) { int T; return sizeof(T); }"
);

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

case!(struct_tag_named_as_typedef, "typedef int S; struct S { int x; };");

case!(union_tag_named_as_typedef, "typedef int S; union S { int x; };");

case!(member_named_as_typedef, "typedef int T; struct S { T T; };");

case!(
    member_access_named_as_typedef,
    "typedef int x; struct S { int x; }; void f(void) { struct S s; s.x = 1; }"
);

case!(typedef_redeclared_as_var, "typedef int T; int T;");

case!(
    knr_struct_def_in_declaration_list,
    "f(a) struct S { int x; } *a; { return 0; }"
);

case!(member_shadows_outer_typedef, "typedef int T; struct S { int T; }; T x;");

case!(enum_tag_named_as_typedef, "typedef int E; enum E { A };");

case!(struct_body_then_typedef_decl, "typedef struct { int x; } S; S s;");

case!(multiword_type_then_typedef, "typedef unsigned long UL; UL u;");

case!(
    typedef_after_nontypedef_decl,
    "typedef int T; int y; void f(void) { y = 1; }"
);

case!(pointer_declarator_typedef, "typedef int T; int *T;");
