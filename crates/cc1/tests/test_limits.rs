#![cfg_attr(rustfmt, rustfmt_skip)]

use std::time::Duration;

use crate::common::{RawRun, compile_raw};

fn compile(name: &str, src: &str) -> RawRun {
    compile_raw(name, format!("{src}\n"))
}

fn chain(kind: &str, n: usize) -> String {
    match kind {
        "sum" => format!("int main(void) {{ int x; x = {}; return x & 255; }}", vec!["1"; n].join("+")),
        "assign" => format!("int main(void) {{ int x; x = {}1; return x; }}", "x = ".repeat(n)),
        "cast" => format!("int main(void) {{ int x = 1; return {}x; }}", "(int)".repeat(n)),
        "parens" => format!("int main(void) {{ return {}1{}; }}", "(".repeat(n), ")".repeat(n)),
        "comma" => format!("int main(void) {{ int x = 0; return ({}); }}", vec!["x"; n].join(",")),
        _ => unreachable!(),
    }
}

fn accepted_at_the_limit(kind: &str) {
    let run = compile(kind, &chain(kind, 32768));
    assert_eq!(run.status, 0, "{kind}: {}", run.stderr);
    assert!(run.stdout.contains("define i32 @main"), "{kind}");
}

fn rejected_beyond_the_limit(kind: &str, n: usize) {
    let run = compile(kind, &chain(kind, n));
    assert_eq!(run.status, 1, "{kind}: {}", run.stderr);
    assert!(run.stderr.contains("expression nesting exceeds the implementation limit of 32768"), "{kind}: {}", run.stderr);
    assert!(!run.stderr.contains("overflowed") && !run.stderr.contains("panicked"), "{kind}: {}", run.stderr);
    assert!(run.stdout.is_empty(), "{kind}");
}

reject!(limit_incomplete_struct_array_unsized, "struct S; struct S a[] = { 1 };");
reject!(limit_unnamed_bitfield_struct_array_unsized, "struct S { int : 3; } a[] = { 1 };");
reject!(limit_incomplete_struct_array_of_arrays, "struct S; struct S a[][2] = { 1 };");
reject!(limit_incomplete_struct_automatic_array, "struct S; int main(void) { struct S a[] = { 1, 2 }; return 0; }");
reject!(limit_incomplete_struct_sized_array, "struct S; struct S a[3] = { 1, 2 };");
reject!(limit_incomplete_struct_nested_braces, "struct S; struct S a[][2] = { { 1 }, { 2 } };");

reject!(limit_struct_over_two_gib, "struct S { char a[0x40000000]; char b[0x40000000]; };");
reject!(limit_struct_over_four_gib, "struct S { char a[0x7fffffff]; char b[0x7fffffff]; char c[0x7fffffff]; };");
reject!(limit_struct_over_four_gib_with_sizeof, "struct S { char a[0x7fffffff]; char b[0x7fffffff]; char c[0x7fffffff]; }; int main(void) { return sizeof(struct S) & 255; }");
reject!(limit_union_just_over_two_gib, "union U { char a[0x7fffffff]; int i; };");
reject!(limit_struct_nested_over_two_gib, "struct A { char a[0x40000000]; }; struct B { struct A x; struct A y; };");
accept!(limit_struct_just_under_two_gib, "struct S { char a[0x7ffffff0]; int i; };");
accept!(limit_union_exactly_max, "union U { char a[0x7ffffffc]; int i; };");
size!(limit_struct_size_near_max, "struct S { char a[0x7ffffff0]; int i; };", "struct S", 0x7ffffff4);

emits!(limit_huge_char_array_tail_is_compact,
    "char a[0x7fffffff] = { 1 }; int main(void) { return a[0]; }",
    "@a = global <{ [1 x i8], [2147483646 x i8] }> <{ [1 x i8] [i8 1], [2147483646 x i8] zeroinitializer }>, align 1");
emits!(limit_int_array_tail_is_compact,
    "int buf[10000000] = { 1 }; int main(void) { return buf[0]; }",
    "@buf = global <{ [1 x i32], [9999999 x i32] }> <{ [1 x i32] [i32 1], [9999999 x i32] zeroinitializer }>, align 4");
emits!(limit_string_tail_is_compact,
    "static char s[50000000] = \"x\"; int main(void) { return s[0]; }",
    "@s = internal global <{ [1 x i8], [49999999 x i8] }> <{ [1 x i8] [i8 120], [49999999 x i8] zeroinitializer }>, align 1");
emits!(limit_all_zero_array_is_zeroinitializer,
    "int z[200] = { 0 }; int main(void) { return z[0]; }",
    "@z = global [200 x i32] zeroinitializer, align 4");
emits!(limit_short_zero_tail_is_not_compacted,
    "int a[3] = { 1 }; int main(void) { return a[0]; }",
    "@a = global [3 x i32] [i32 1, i32 zeroinitializer, i32 zeroinitializer], align 4");
emits!(limit_tail_at_threshold_is_compact,
    "int a[65] = { 1 }; int main(void) { return a[0]; }",
    "@a = global <{ [1 x i32], [64 x i32] }> <{ [1 x i32] [i32 1], [64 x i32] zeroinitializer }>, align 4");
emits!(not limit_tail_below_threshold_is_expanded,
    "int a[64] = { 1 }; int main(void) { return a[0]; }",
    "<{ [1 x i32]");
emits!(limit_automatic_string_array_uses_aligned_constant,
    "int main(void) { char buf[100000] = \"x\"; return buf[0]; }",
    " = private unnamed_addr constant <{ [1 x i8], [99999 x i8] }> <{ [1 x i8] [i8 120], [99999 x i8] zeroinitializer }>, align 1");

exits!(limit_big_int_array_short_initializer,
    "int buf[1000000] = { 7 }; int main(void) { return buf[0] + buf[999999] + 1; }", 8);
exits!(limit_big_string_array_short_initializer,
    "static char s[100000] = \"x\"; int main(void) { return s[0] + s[99999] + s[1]; }", 120);
exits!(limit_big_wide_string_array,
    "long w[1000] = L\"ab\"; int main(void) { return w[0] + w[1] + w[2] + w[999]; }", 195);
exits!(limit_big_member_array_tail,
    "struct S { int a; char name[1000]; int tail[200]; short z; } g = { 1, \"hello\", { 1, 2, 3 }, 9 };
     int main(void) { return g.a + g.name[1] + g.name[999] + g.tail[2] + g.tail[199] + g.z; }", 114);
exits!(limit_big_two_dimensional_array,
    "int m[100][100] = { { 1, 2 }, { 3 } }; int n[100][3] = { { 1 }, { 2 }, { 3 } };
     int main(void) { return m[0][1] + m[1][0] + m[99][99] + n[2][0] + n[99][2] + m[1][1]; }", 8);
exits!(limit_big_array_of_structs_with_union_member,
    "union U { char c[300]; int i; }; struct T { int k; union U u; char tag[200]; };
     struct T ts[100] = { { 1, { \"abc\" }, \"zz\" }, { 2 } };
     int main(void) { return ts[0].k + ts[0].u.c[1] + ts[1].k + ts[0].tag[1] + ts[99].k + ts[1].u.c[0] + ts[50].tag[3]; }", 223);
exits!(limit_big_union_initialized_by_string,
    "union V { char c[300]; int i; } v = { \"q\" }; int main(void) { return v.c[0] + v.c[299]; }", 113);
exits!(limit_big_array_of_pointers_and_doubles,
    "char *pp[100] = { \"a\", 0 }; double dd[100] = { 1.5 };
     int main(void) { return pp[0][0] + (int)dd[0] + (pp[1] == 0) + (dd[99] == 0.0); }", 100);
exits!(limit_big_array_of_bitfield_structs,
    "struct B { int a : 3; int b : 5; char x[100]; } bb[70] = { { 1, 2, \"hi\" } };
     int main(void) { return bb[0].a + bb[0].b * 8 + bb[0].x[1] + bb[69].a + bb[69].x[99]; }", 122);
exits!(limit_unsized_array_keeps_exact_length,
    "int ex[] = { 1, 2, 3 }; int main(void) { return sizeof(ex); }", 12);
exits!(limit_unsized_string_array_keeps_exact_length,
    "char ex[] = \"abc\"; int main(void) { return sizeof(ex); }", 4);
exits!(limit_automatic_big_string_array,
    "int main(void) { char buf[100000] = \"x\"; int a[200] = { 5 }; struct { char s[100]; int v[100]; } t = { \"ab\", { 1 } };
     return buf[0] + buf[99999] + a[0] + a[199] + t.s[1] + t.v[0] + t.v[99]; }", 224);
exits!(limit_huge_char_array_runs,
    "char a[0x7fffffff] = { 1 }; int main(void) { return a[0]; }", 1);

test_case!(depth_sum_at_the_limit, { accepted_at_the_limit("sum") });
test_case!(depth_assign_at_the_limit, { accepted_at_the_limit("assign") });
test_case!(depth_parens_at_the_limit, { accepted_at_the_limit("parens") });
test_case!(depth_comma_list_is_flat, { accepted_at_the_limit("comma") });

test_case!(depth_sum_beyond_the_limit, { rejected_beyond_the_limit("sum", 150000) });
test_case!(depth_assign_beyond_the_limit, { rejected_beyond_the_limit("assign", 32771) });
test_case!(depth_parens_just_beyond_the_limit, { rejected_beyond_the_limit("parens", 32770) });

test_case!(depth_diagnostic_points_into_the_expression, {
    let run = compile("point", &chain("sum", 40000));
    assert!(run.stderr.contains(".c:1:"), "{}", run.stderr);
    assert_eq!(run.stderr.matches("error:").count(), 1, "{}", run.stderr);
});

test_case!(cast_chain_compiles_in_linear_time, {
    let run = compile("casts", &chain("cast", 20000));
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert!(run.elapsed < Duration::from_secs(20), "{:?}", run.elapsed);
});

test_case!(constant_cast_chain_compiles_in_linear_time, {
    let src = format!("enum {{ A = {}1 }};\nint main(void) {{ return A; }}", "(int)".repeat(20000));
    let run = compile("constcasts", &src);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert!(run.elapsed < Duration::from_secs(20), "{:?}", run.elapsed);
});

reject!(limit_fold_probe_cache_keeps_variable_array_size_error, "int x; int a[x];");
reject!(limit_fold_probe_cache_keeps_variable_array_size_after_cast, "int main(void) { int x = 1; int a[(int)(int)x]; return 0; }");
reject!(limit_fold_probe_cache_keeps_nonconstant_enumerator, "int x; enum { A = (int)(int)x };");
reject!(limit_fold_probe_cache_keeps_nonconstant_case, "int main(void) { int x = 1; switch (x) { case (int)(int)x: return 0; } return 1; }");
