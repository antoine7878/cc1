use crate::common::Unit;

folded!(fold_addition, "int main(void) { int a; a = 1 + 1; return a; }", ["Int(2)"]);
folded!(fold_nested_operations, "int main(void) { return (2 * 3) + (4 - 1); }", ["Int(6)", "Int(3)", "Int(9)"]);
folded!(
    fold_unary,
    "int main(void) { return -(3) + ~0 + !5; }",
    ["Int(-3)", "Int(-1)", "Int(-4)", "Int(0)", "Int(-4)"]
);
folded!(fold_comparison, "int main(void) { return 1 < 2; }", ["Int(1)"]);
folded!(fold_to_result_type, "int main(void) { long l; l = 1 + 1L; return 0; }", ["Long(2)"]);
folded!(fold_unsigned, "int main(void) { unsigned u; u = 1u + 2; return 0; }", ["UnsignedInt(3)"]);
folded!(fold_floating, "int main(void) { double d; d = 1.5 + 2.5; return 0; }", ["Double(4.0)"]);
folded!(fold_cast, "int main(void) { return (char)300; }", ["Int(44)"]);
folded!(fold_enum_variant, "enum E { A = 2 }; int main(void) { return A + 1; }", ["Int(2)", "Int(2)", "Int(3)"]);
folded!(fold_ternary, "int main(void) { return 1 ? 2 : 3; }", ["Int(2)"]);

folded!(fold_skips_unevaluated_and_operand, "int f(void); int main(void) { return 0 && f(); }", ["Int(0)"]);
folded!(fold_skips_unevaluated_or_operand, "int f(void); int main(void) { return 1 || f(); }", ["Int(1)"]);
folded!(fold_skips_unevaluated_ternary_branch, "int f(void); int main(void) { return 0 ? f() : 3; }", ["Int(3)"]);
folded!(fold_stops_at_call, "int f(void); int main(void) { return 1 && f(); }", []);
folded!(fold_stops_at_variable, "int main(void) { int x; x = 1; return x + 1; }", []);
folded!(fold_stops_at_assignment, "int main(void) { int x; return (x = 1) + 1; }", []);
folded!(fold_stops_at_comma, "int main(void) { return (1, 2); }", []);
folded!(fold_leaves_division_by_zero_to_run_time, "int main(void) { int a; a = 1 / 0; return 0; }", []);
folded!(fold_leaves_modulo_by_zero_to_run_time, "int main(void) { int a; a = 1 % 0; return 0; }", []);
folded!(
    fold_leaves_floating_cast_of_an_operation_alone,
    "int main(void) { return (int)(1.5 + 1.0); }",
    ["Double(2.5)"]
);

#[test]
fn fold_emits_the_value_instead_of_the_operation() {
    let src = "int main(void) { int a; a = 40 + 2; return a; }";
    let unit = Unit::compile(src);
    assert!(unit.accepts(), "{}", unit.render());
    let ir = unit.ir();
    assert!(ir.contains("store i32 42, ptr"), "{ir}");
    assert!(!ir.contains("add"), "{ir}");
}

#[test]
fn fold_keeps_the_run_time_division_by_zero() {
    let src = "int main(void) { int a; a = 1 / 0; return 0; }";
    let unit = Unit::compile(src);
    assert!(unit.accepts(), "{}", unit.render());
    let ir = unit.ir();
    assert!(ir.contains("sdiv i32 1, 0"), "{ir}");
}

exits!(folded_local_assignment, "int main(void) { int a; a = 40 + 2; return a; }", 42);
exits!(folded_short_circuit_skips_the_call, "int f(void) { return 1; } int main(void) { return (0 && f()) + 42; }", 42);
exits!(folded_ternary_skips_the_call, "int f(void) { return 1; } int main(void) { return 0 ? f() : 42; }", 42);
exits!(folded_long_operation, "int main(void) { long l; l = 20L + 22; return l; }", 42);
