#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(folded_local_assignment, "int main(void) { int a; a = 40 + 2; return a; }", 42, "");
valid!(folded_short_circuit_skips_the_call, "int f(void) { return 1; } int main(void) { return (0 && f()) + 42; }", 42, "");
valid!(folded_ternary_skips_the_call, "int f(void) { return 1; } int main(void) { return 0 ? f() : 42; }", 42, "");
valid!(folded_long_operation, "int main(void) { long l; l = 20L + 22; return l; }", 42, "");
valid!(fold_addition, "int main(void) { int a; a = 1 + 1; return a; }", 2, "");
valid!(fold_nested_operations, "int main(void) { return (2 * 3) + (4 - 1); }", 9, "");
valid!(fold_unary, "int main(void) { return -(3) + ~0 + !5; }", 252, "");
valid!(fold_comparison, "int main(void) { return 1 < 2; }", 1, "");
valid!(fold_to_result_type, "int main(void) { long l = 1 + 1L; return !(l == 2L); }", 0, "");
valid!(fold_unsigned, "int main(void) { unsigned u = 1u + 2; return !(u == 3u); }", 0, "");
valid!(fold_floating, "int main(void) { double d = 1.5 + 2.5; return !(d == 4.0); }", 0, "");
valid!(fold_long_double_at_x87_precision, "int main(void) { return (9007199254740992.0L + 1.0L != 9007199254740992.0L) + 41; }", 42, "");
valid!(fold_cast, "int main(void) { return (char)300; }", 44, "");
valid!(fold_enumerator, "enum E { A = 2 }; int main(void) { return A + 1; }", 3, "");
valid!(fold_ternary, "int main(void) { return 1 ? 2 : 3; }", 2, "");
valid!(fold_skips_unevaluated_and_operand, "int f(void); int main(void) { return 0 && f(); }", 0, "");
valid!(fold_skips_unevaluated_or_operand, "int f(void); int main(void) { return 1 || f(); }", 1, "");
valid!(fold_skips_unevaluated_ternary_branch, "int f(void); int main(void) { return 0 ? f() : 3; }", 3, "");
valid!(fold_stops_at_call, "int f(void); int main(void) { return 1 && f(); }\nint f(void) { return 1; }", 1, "");
valid!(fold_stops_at_variable, "int main(void) { int x; x = 1; return x + 1; }", 2, "");
valid!(fold_stops_at_assignment, "int main(void) { int x; return (x = 1) + 1; }", 2, "");
valid!(fold_stops_at_comma, "int main(void) { return (1, 2); }", 2, "");
valid!(fold_leaves_division_by_zero_to_run_time, "int fixture_main(void) { int a; a = 1 / 0; return 0; }\nint main(void) { return 0; }", 0, "");
valid!(fold_leaves_modulo_by_zero_to_run_time, "int fixture_main(void) { int a; a = 1 % 0; return 0; }\nint main(void) { return 0; }", 0, "");
