#![cfg_attr(rustfmt, rustfmt_skip)]
use crate::common::compile_raw;

fn run_raw(name: &str, src: &str) -> (i32, String) {
    let run = compile_raw(name, src);
    (run.status, run.stderr)
}

reject!(stray_hash_after_expression, "int main(void) { return 0 # 1\n; }");
reject!(stray_hash_after_initializer, "int x = 1; int y = x # this is swallowed ; int z;");
reports!(stray_hash_is_reported, "int x = 1 # 2;", ["<stdin>:1:11: error: stray '#' in program", "<stdin>:1:13: error: syntax error, unexpected CONSTANT, expecting ',' or ';'"]);
test_case!(stray_hash_in_raw_input_is_rejected, {
    let (status, stderr) = run_raw("stray", "int main(void) { return 0 # 1\n; }\n");
    assert_eq!(status, 1, "{stderr}");
    assert!(stderr.contains("stray '#' in program"), "{stderr}");
});
test_case!(directive_line_in_raw_input_is_skipped, {
    let (status, stderr) = run_raw("hello", "#include <stdio.h>\nint main()\n{\nprintf(\"hello, world\\n\");\n}\n");
    assert_eq!(status, 0, "{stderr}");
});
test_case!(indented_directive_line_in_raw_input_is_skipped, {
    let (status, stderr) = run_raw("indented", "  #pragma foo\n\t# define X\nint main(void){ return 0; }\n");
    assert_eq!(status, 0, "{stderr}");
});

reject!(case_conditional_with_non_constant_arm, "int main(void){ int x = 1; switch (x) { case 1 ? 2 : x: break; } return 0; }");
reject!(case_logical_and_with_non_constant_operand, "int main(void){ int x = 1; switch (x) { case 0 && x: break; } return 0; }");
reject!(case_logical_or_with_non_constant_operand, "int main(void){ int x = 1; switch (x) { case 1 || x: break; } return 0; }");
reject!(array_size_conditional_with_non_constant_arm, "int main(void){ int x = 1; int a[1 ? 2 : x]; return 0; }");
reject!(enumerator_logical_and_with_non_constant_operand, "int x; enum { A = 0 && x };");
accept!(static_initializer_logical_or_non_constant, "int x; int y = 1 || x; int main(void){ return 0; }");
accept!(static_initializer_logical_and_non_constant, "int x; int y = 0 && x; int main(void){ return 0; }");
accept!(static_initializer_conditional_non_constant, "int x; int y = 1 ? 2 : x; int main(void){ return 0; }");
accept!(case_unevaluated_division_by_zero, "int main(void){ int x = 1; switch (x) { case 0 && (1 / 0): break; case 1 ? 3 : 1 / 0: break; } return 0; }");
accept!(case_unevaluated_sizeof_operand, "int main(void){ int x = 1; switch (x) { case 0 || sizeof(x): break; } return 0; }");
exits!(runtime_logical_and_still_folds, "int f(void) { return 1; } int main(void) { return 0 && f() ? 1 : 42; }", 42);

reports!(enumerator_float_overflow, "enum { A = (int)1e10 };", ["<test>:1:12: error: overflow in constant expression"]);
reject!(case_float_overflow, "int main(void){ int x = 1; switch (x) { case (int)1e10: ; } return 0; }");
reject!(static_initializer_float_overflow, "int i = (int)1e10;");
reject!(static_unsigned_initializer_negative_float, "unsigned u = (unsigned)-1.0;");
accept!(static_initializer_float_in_range, "int a = (int)2147483647.5; unsigned b = (unsigned)4294967295.5; int c = (int)-2147483648.5;");
exits!(block_scope_float_conversion_is_runtime, "int main(void){ double d = 3.9; int i = (int)d; int j = (int)1e5; return i + (j == 100000) * 39; }", 42);
accept!(block_scope_float_overflow_initializer, "int main(void){ int i = (int)1e10; return 0; }");

test_case!(float_constant_out_of_range_warns, {
    let unit = crate::common::Unit::compile("double d = 1e400; float f = 1e39f; long double e = 1e5000L;");
    assert!(unit.accepts());
    assert_eq!(unit.messages(), [
        "<test>:1:12: warning: floating constant exceeds range of 'double'",
        "<test>:1:29: warning: floating constant exceeds range of 'float'",
        "<test>:1:52: warning: floating constant exceeds range of 'long double'",
    ]);
});
test_case!(float_constant_truncated_to_zero_warns, {
    let unit = crate::common::Unit::compile("double z = 1e-400; float g = 1e-46f; long double h = 1e-5000L;");
    assert!(unit.accepts());
    assert_eq!(unit.messages(), [
        "<test>:1:12: warning: floating constant truncated to zero",
        "<test>:1:30: warning: floating constant truncated to zero",
        "<test>:1:54: warning: floating constant truncated to zero",
    ]);
});
test_case!(float_constant_boundaries_warn, {
    let unit = crate::common::Unit::compile("float a = 3.5e38f; double b = 1.8e308; long double c = 1.2e4932L; double d = 2.4e-324; float e = 7e-46f;");
    assert_eq!(unit.messages().len(), 5);
});
accept!(float_constant_in_range_is_clean, "float a = 3.4028235e38f; double b = 1.7976931348623157e308; long double c = 1.18973149535723176502e4932L; double d = 4.9e-324; float e = 1.4e-45f; double z = 0.0; double y = 0e-400; double w = 2.5e-324; float v = 8e-46f;");

reject!(array_typedef_const_duplicated, "typedef const int A[2]; const A x = {1, 2};");
reject!(array_typedef_volatile_duplicated, "typedef volatile int A[2]; volatile A x;");
reject!(array_typedef_const_duplicated_2d, "typedef const int A[2][3]; const A x = {{0}};");
accept!(array_typedef_other_qualifier_ok, "typedef const int A[2]; volatile A z = {0};");
accept!(array_typedef_unqualified_element_ok, "typedef int A[2]; const A x = {1, 2}; typedef int B[2][3]; volatile B y;");

reject!(address_of_void_object, "extern void v; void *p = &v;");
reject!(assign_to_void_object, "extern void v; int main(void){ v = 0; return 0; }");
accept!(void_object_as_expression_statement, "extern void v; int main(void){ v; return 0; }");
accept!(const_void_object_declaration, "extern const void cv; int main(void){ return 0; }");
