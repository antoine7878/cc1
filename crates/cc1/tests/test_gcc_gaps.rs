use std::fs;

use cc1::context::Context;
use cc1::parser::parse_source;
use cc1::semantic::Analyzer;
use libft::TmpDir;

use crate::common::{Unit, strip_ansi};

fn compile_bytes(dir: &TmpDir, bytes: &[u8]) -> Unit {
    let path = dir.join("src.c");
    fs::write(&path, bytes).unwrap();
    let mut ctx = Context::default();
    ctx.set_file_name(path.to_str().unwrap().to_string());
    let ctx = parse_source(ctx);
    let sema = Analyzer::analyze(ctx);
    Unit { ctx: cc1::context::ctx(), sema: Some(sema), status: 0 }
}

fn enumerator(unit: &Unit, name: &str) -> Option<String> {
    unit.enumerators().into_iter().find(|(n, _)| n == name).map(|(_, v)| v)
}

exits!(gap_arrow_on_array, "struct S { int a; } arr[2]; int main(void) { arr->a = 42; return arr[0].a; }", 42);

exits!(gap_incomplete_extern_object, "struct S; extern struct S s; int main(void) { return 0; }", 0);

exits!(gap_incomplete_union_extern_object, "union U; extern union U u; int main(void) { return 0; }", 0);

exits!(gap_incomplete_return_declaration, "struct S; struct S f(void); int main(void) { return 0; }", 0);

reject!(gap_nested_tag_redefinition, "struct S { struct S { int a; } x; };");

emits!(not gap_string_concat_emits_no_intermediate_literals, "char *p = \"ab\" \"cd\" \"ef\";", "@.str.1");

exits!(
    gap_paren_comma_first_argument,
    "int f(int a, int b) { return a * 10 + b; } int main(void) { return f((1, 2), 3); }",
    23
);

exits!(
    gap_paren_comma_single_argument_unprototyped,
    "int g(); int main(void) { return g((1, 2)); } int g(a) int a; { return a; }",
    2
);

reject!(
    gap_paren_comma_whole_argument_list,
    "int printf(const char *, ...); int main(void) { printf((\"%d\", 1)); return 0; }"
);

exits!(
    gap_long_divided_by_unsigned_bit_field,
    "struct B { unsigned x : 4; } b; int main(void) { long l = -6; b.x = 2; return l / b.x == -3; }",
    1
);

exits!(
    gap_long_compared_to_unsigned_bit_field,
    "struct B { unsigned x : 21; } b; int main(void) { long l = -5; b.x = 3; return l <= b.x; }",
    1
);

exits!(
    gap_conditional_long_and_unsigned_bit_field,
    "struct B { unsigned x : 4; } b; int main(void) { long l = -1; b.x = 1; return (0 ? b.x : l) < 0; }",
    1
);

exits!(gap_enum_without_negative_is_unsigned, "enum E { A, B }; int main(void) { enum E x = A; return x - 1 < 0; }", 0);

accept!(gap_enum_without_negative_points_as_unsigned, "enum E { A, B } e; unsigned int *p = &e;");

reject!(gap_enum_without_negative_is_not_int, "enum E { A, B } e; int *p = &e;");

#[test]
fn gap_latin1_bytes_in_literals() {
    let dir = TmpDir::new("cc1-gap-latin1");
    let unit = compile_bytes(&dir, b"enum { N = sizeof \"\xe9t\xe9\", C = '\xe9' };\n");
    assert_eq!(enumerator(&unit, "N").as_deref(), Some("4"), "{}", unit.render());
    assert_eq!(enumerator(&unit, "C").as_deref(), Some("-23"), "{}", unit.render());
}

#[test]
fn gap_wide_literal_decodes_utf8() {
    let dir = TmpDir::new("cc1-gap-wide-utf8");
    let unit = compile_bytes(&dir, "enum { N = sizeof L\"\u{e9}\", C = L'\u{e9}' };\n".as_bytes());
    assert_eq!(enumerator(&unit, "N").as_deref(), Some("8"), "{}", unit.render());
    assert_eq!(enumerator(&unit, "C").as_deref(), Some("233"), "{}", unit.render());
}

exits!(
    gap_pragma_pack,
    "#pragma pack(1)\nstruct S { char c; int i; };\n#pragma pack()\nint main(void) { return sizeof(struct S); }",
    5
);

exits!(gap_float_constant_excess_precision_compare, "double g = 55.1; int main(void) { return g != 55.1; }", 1);

exits!(
    gap_float_constant_excess_precision_fold,
    "double d = 0.1 + 0.2; double e = 0.3; int main(void) { return d == e; }",
    1
);

exits!(
    gap_int_to_float_conversion_rounds,
    "int main(void) { volatile int i = 16777217; float f = (float)i; return f == i; }",
    1
);

exits!(
    gap_enum_constant_in_parameter_list,
    "int f(enum E { A, B } x) { return x + B; } int main(void) { return f(0); }",
    1
);

exits!(warns gap_deref_void_pointer_discarded, "int main(void) { int x = 1; void *p = &x; *p; (void)*p; return 0; }", 0);

exits!(
    gap_typedef_void_parameter_list,
    "typedef void V; int f(V); int f(V) { return 3; } int main(void) { return f(); }",
    3
);

accept!(gap_extern_void_object, "extern void v;");

reject!(gap_hex_pp_number_with_sign, "int x = 0x1E-1;");

reject!(gap_stray_hash_outside_directive, "int x = 1; # garbage\nint y;");

reject!(gap_cpp_comment, "int x = 4 // 2\n;");

reject!(gap_digraphs, "int a<:2:>;");

reject!(gap_newline_in_string_literal, "char *s = \"a\nb\";");

#[test]
fn gap_string_literal_longer_than_509() {
    let src = format!("char s[] = \"{}\";", "x".repeat(510));
    crate::common::run_reject("gap_string_literal_longer_than_509", &src);
}

reject!(gap_sizeof_type_name_then_operand, "int f(void) { return sizeof (int) 1; }");

reject!(gap_identifier_list_in_declaration, "int f(a, b);");

reject!(gap_qualified_function_type, "typedef int F(void); const F f;");

reject!(gap_static_incomplete_array, "static int a[];");

reject!(gap_main_returning_void, "void main(void) { }");

reject!(gap_main_char_parameter, "int main(char c) { return 0; }");

reject!(gap_main_static, "static int main(void) { return 0; }");

reject!(gap_subscript_non_lvalue_array, "struct S { int a[3]; }; struct S f(void); int g(void) { return f().a[1]; }");

reject!(gap_decay_non_lvalue_array, "struct S { int a[3]; } x, y; int g(void) { int *p = (x = y).a; return p != 0; }");

reject!(gap_register_array_subscript, "int f(void) { register int a[3]; a[0] = 1; return a[0]; }");

reject!(gap_register_array_decay, "int f(void) { register int a[3]; int *p = a; return p != 0; }");

reject!(
    gap_register_struct_member_address,
    "struct S { int x; }; int f(void) { register struct S s; int *p = &s.x; return p != 0; }"
);

reject!(gap_function_to_object_pointer_cast, "int f(void); void g(void) { void *v = (void *)f; }");

reject!(gap_object_to_function_pointer_cast, "void g(void) { int x; int (*p)(void) = (int (*)(void))&x; }");

reject!(
    gap_conditional_merges_qualifiers_into_void_pointer,
    "void g(int c) { int x = 1; const int *p = &x; void *q = &x; void *r = c ? p : q; }"
);

reject!(gap_deref_incomplete_struct, "struct S; struct S *p; void f(void) { *p; }");

reject!(gap_call_qualified_void_function, "void volatile f(); void g(void) { f(); }");

reject!(gap_static_init_pointer_to_char, "int g; char c = (char)&g;");

#[test]
fn gap_constant_shift_too_wide() {
    let unit = Unit::compile("enum { A = 1 << 32 };");
    assert!(!unit.accepts() && !unit.only_warns(), "should be an error:\n{}", unit.render());
}

#[test]
fn gap_constant_shift_negative() {
    let unit = Unit::compile("enum { B = 1 << -1 };");
    assert!(!unit.accepts() && !unit.only_warns(), "should be an error:\n{}", unit.render());
}

reject!(gap_array_from_parenthesized_string, "char s[] = (\"abc\");");

reject!(gap_struct_too_large, "struct S { char a[0x40000000]; char b[0x40000000]; };");
