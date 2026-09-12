exits!(return_constant, "int main(void) { return 42; }", 42);
exits!(return_zero, "int main(void) { return 0; }", 0);
exits!(return_negative_wraps, "int main(void) { return -1; }", 255);

exits!(add, "int main(void) { return 40 + 2; }", 42);
exits!(sub, "int main(void) { return 50 - 8; }", 42);
exits!(mul, "int main(void) { return 6 * 7; }", 42);
exits!(div, "int main(void) { return 84 / 2; }", 42);
exits!(rem, "int main(void) { return 142 % 100; }", 42);
exits!(shifts, "int main(void) { return (1 << 6) - (44 >> 1); }", 42);
exits!(bitwise, "int main(void) { return (0x7f & 0x2a) | (0x0 ^ 0x0); }", 42);
exits!(precedence, "int main(void) { return 2 + 4 * 10; }", 42);
exits!(unary_minus, "int main(void) { return -(-42); }", 42);
exits!(bit_not, "int main(void) { return ~(-43); }", 42);
exits!(logical_not, "int main(void) { return !0 + !5 + 41; }", 42);
exits!(
    comparisons,
    "int main(void) { return (1 < 2) + (2 <= 2) + (3 > 2) + (3 >= 3) + (4 == 4) + (4 != 5) + 36; }",
    42
);
exits!(logical_and_or, "int main(void) { return (1 && 2) + (0 || 3) + 40; }", 42);
exits!(ternary, "int main(void) { return 1 ? 42 : 7; }", 42);
exits!(ternary_false, "int main(void) { return 0 ? 7 : 42; }", 42);
exits!(comma, "int main(void) { return (7, 42); }", 42);

exits!(local_variable, "int main(void) { int x; x = 42; return x; }", 42);
exits!(local_arithmetic, "int main(void) { int a; int b; a = 6; b = 7; return a * b; }", 42);
exits!(compound_assign, "int main(void) { int x; x = 40; x += 2; return x; }", 42);
exits!(
    compound_assign_all,
    "int main(void) { int x; x = 1; x <<= 6; x -= 20; x *= 2; x /= 2; x |= 2; x &= 0xff; x ^= 0; x %= 100; x >>= 0; return x; }",
    46
);
exits!(pre_increment, "int main(void) { int x; x = 41; return ++x; }", 42);
exits!(post_increment, "int main(void) { int x; x = 42; return x++; }", 42);
exits!(post_increment_side_effect, "int main(void) { int x; x = 41; x++; return x; }", 42);
exits!(pre_decrement, "int main(void) { int x; x = 43; return --x; }", 42);
exits!(post_decrement, "int main(void) { int x; x = 43; x--; return x; }", 42);
exits!(assign_chain, "int main(void) { int a; int b; a = b = 42; return a + b - 42; }", 42);
exits!(char_local, "int main(void) { char c; c = 'A'; return c - 23; }", 42);
exits!(unsigned_local, "int main(void) { unsigned u; u = 42u; return u; }", 42);
exits!(long_local, "int main(void) { long l; l = 42L; return l; }", 42);
exits!(unsigned_division, "int main(void) { unsigned u; u = 0xffffffffu; return u / 0x06185ea6u; }", 42);
exits!(signed_division_negative, "int main(void) { int a; a = -85; return a / -2; }", 42);
exits!(signed_remainder_negative, "int main(void) { int a; a = -85; return -(a % 43); }", 42);
exits!(arithmetic_shift, "int main(void) { int a; a = -168; return -(a >> 2); }", 42);
exits!(logical_shift, "int main(void) { unsigned a; a = 0xa8000000u; return a >> 26; }", 42);

// exits!(ignore "call ptr instead of call <ret ty>", call_in_expression, "int f(void) { return 6; } int g(void) { return 7; } int main(void) { return f() * g(); }", 42);
// exits!(ignore "call ptr instead of call <ret ty>", call_prototype_first, "int f(void); int main(void) { return f(); } int f(void) { return 42; }", 42);
// exits!(ignore "call arguments", call_two_arguments, "int f(int a, int b) { return a * b; } int main(void) { return f(6, 7); }", 42);
// exits!(ignore "call arguments", call_char_argument, "int f(char c) { return c; } int main(void) { return f(42); }", 42);
// exits!(ignore "call arguments", call_short_argument, "int f(short s) { return s; } int main(void) { return f(42); }", 42);
// exits!(ignore "call arguments", recursion, "int fact(int n) { return n ? n * fact(n - 1) : 1; } int main(void) { return fact(5) - 78; }", 42);

exits!(if_true, "int main(void) { if (1) return 42; return 7; }", 42);
// exits!(ignore "control flow", if_false, "int main(void) { if (0) return 7; return 42; }", 42);
// exits!(ignore "control flow", if_else, "int main(void) { if (0) return 7; else return 42; }", 42);
// exits!(ignore "control flow", while_loop, "int main(void) { int i; i = 0; while (i < 42) i++; return i; }", 42);
// exits!(ignore "control flow", do_while, "int main(void) { int i; i = 0; do i++; while (i < 42); return i; }", 42);
// exits!(ignore "control flow", for_loop, "int main(void) { int i; int s; s = 0; for (i = 0; i < 7; i++) s += 6; return s; }", 42);
// exits!(ignore "control flow", break_loop, "int main(void) { int i; for (i = 0; ; i++) if (i == 42) break; return i; }", 42);
// exits!(ignore "control flow", continue_loop, "int main(void) { int i; int s; s = 0; for (i = 0; i < 10; i++) { if (i % 2) continue; s += i; } return s + 22; }", 42);
// exits!(ignore "control flow", goto_label, "int main(void) { int i; i = 0; again: i++; if (i < 42) goto again; return i; }", 42);
// exits!(ignore "switch", switch_case, "int main(void) { switch (2) { case 1: return 1; case 2: return 42; default: return 3; } }", 42);
// exits!(ignore "switch", switch_default, "int main(void) { switch (9) { case 1: return 1; default: return 42; } }", 42);
// exits!(ignore "switch", switch_fallthrough, "int main(void) { int x; x = 40; switch (1) { case 1: x++; case 2: x++; break; case 3: x = 0; } return x; }", 42);
exits!(nested_blocks, "int main(void) { int x; x = 1; { int x; x = 7; } return x + 41; }", 42);

exits!(short_circuit_and, "int f(void) { return 0; } int main(void) { int x; x = 42; (0 && (x = 1)); return x; }", 42);
exits!(short_circuit_or, "int main(void) { int x; x = 42; (1 || (x = 1)); return x; }", 42);
exits!(int_wraparound, "int main(void) { unsigned u; u = 0xffffffffu; u += 43; return u; }", 42);
exits!(mixed_signedness, "int main(void) { unsigned u; int i; u = 1; i = -1; return (i < u) ? 42 : 7; }", 7);
exits!(long_arithmetic, "int main(void) { long a; a = 100000L; return (a * 42) / 100000L; }", 42);
exits!(sizeof_int, "int main(void) { return sizeof(int) * 10 + 2; }", 42);
exits!(sizeof_expr, "int main(void) { long l; return sizeof l * 10 + 2; }", 42);
exits!(cast, "int main(void) { return (int) 42.9; }", 42);
// exits!(ignore "globals", global_variable, "int g; int main(void) { g = 42; return g; }", 42);
// exits!(ignore "globals", global_initialized, "int g = 42; int main(void) { return g; }", 42);
// exits!(ignore "globals", static_local, "int f(void) { static int n = 40; return ++n; } int main(void) { f(); return f(); }", 42);
// exits!(ignore "aggregates", struct_member, "struct s { int a; int b; }; int main(void) { struct s v; v.a = 40; v.b = 2; return v.a + v.b; }", 42);
exits!(enum_constant, "enum e { A = 40, B }; int main(void) { return B + 1; }", 42);

exits!(short_local, "int main(void) { short s; s = 1000; return s / 24 + 1; }", 42);
exits!(char_promotion, "int main(void) { char c; c = 100; return (c + c) / 5 + 2; }", 42);
exits!(unsigned_char_promotion, "int main(void) { unsigned char c; c = 200; return c / 5 + 2; }", 42);
exits!(char_increment, "int main(void) { char c; c = 41; c++; return c; }", 42);
exits!(compound_assign_char, "int main(void) { char c; c = 40; c += 2; return c; }", 42);
exits!(void_cast_local, "int main(void) { int x; x = 42; (void) x; return x; }", 42);
exits!(pointer_deref, "int main(void) { int x; int *p; x = 42; p = &x; return *p; }", 42);
exits!(call_return, "int f(void) { return 42; } int main(void) { return f(); }", 42);

exits!(comparison_local, "int main(void) { int a; int b; a = 1; b = 2; return (a < b) + 41; }", 42);
exits!(logical_not_local, "int main(void) { int x; x = 0; return !x + 41; }", 42);
exits!(logical_and_or_local, "int main(void) { int a; int b; a = 1; b = 2; return (a && b) + (a || 0) + 40; }", 42);
exits!(logical_nested_rhs, "int main(void) { int a; int b; a = 1; b = 0; return (a && (b || a)) + 41; }", 42);
exits!(ternary_local, "int main(void) { int c; c = 1; return c ? 42 : 7; }", 42);
exits!(ternary_nested_arm, "int main(void) { int c; c = 1; return c ? (c ? 42 : 1) : 7; }", 42);
exits!(ternary_common_type, "int main(void) { int c; long l; c = 1; l = 42; return c ? l : 0; }", 42);

exits!(pointer_condition, "int main(void) { int x; int *p; p = &x; return p ? 42 : 7; }", 42);
exits!(pointer_logical_not, "int main(void) { int x; int *p; p = &x; return !p + 42; }", 42);
exits!(pointer_logical_and, "int main(void) { int x; int *p; p = &x; return (p && 1) + 41; }", 42);
exits!(double_condition, "int main(void) { double d; d = 0.5; return d ? 42 : 7; }", 42);

exits!(pointer_store, "int main(void) { int x; int *p; p = &x; *p = 42; return x; }", 42);
exits!(compound_assign_through_pointer, "int main(void) { int x; int *p; x = 40; p = &x; *p += 2; return x; }", 42);
exits!(pointer_target_increment, "int main(void) { int x; int *p; x = 41; p = &x; (*p)++; return x; }", 42);
exits!(compound_assign_int_double, "int main(void) { int i; double d; i = 40; d = 2.5; i += d; return i; }", 42);

exits!(pointer_add, "int main(void) { int a[2]; int *p; a[1] = 42; p = a; return *(p + 1); }", 42);
exits!(pointer_difference, "int main(void) { int a[3]; int *p; int *q; p = a; q = &a[2]; return (q - p) + 40; }", 42);
exits!(pointer_comparison, "int main(void) { int a[2]; int *p; int *q; p = a; q = &a[1]; return (p < q) + (p == q) + 41; }", 42);
exits!(pointer_increment, "int main(void) { int a[2]; int *p; a[0] = 1; a[1] = 42; p = a; p++; return *p; }", 42);

exits!(array_index, "int main(void) { int a[3]; a[0] = 40; a[1] = 2; a[2] = a[0] + a[1]; return a[2]; }", 42);
exits!(array_variable_index, "int main(void) { int a[3]; int i; i = 1; a[1] = 42; return a[i]; }", 42);
exits!(pointer_subscript, "int main(void) { int a[2]; int *p; a[1] = 42; p = a; return p[1]; }", 42);
exits!(double_array_index, "int main(void) { double a[2]; a[0] = 1.0; a[1] = 41.0; return a[0] + a[1]; }", 42);
exits!(string_literal_char, "int main(void) { return \"*\"[0]; }", 42);

exits!(null_pointer_initializer, "int main(void) { int *p = 0; int x; x = 42; return x; }", 42);
exits!(null_pointer_assign, "int main(void) { int *p; int x; p = 0; x = 42; return x; }", 42);
exits!(null_pointer_cast, "int main(void) { int *p; int x; p = (int *) 0; x = 42; return x; }", 42);
exits!(char_to_pointer_sign_extends, "int main(void) { char c; c = -1; return ((int) (char *) c >> 8) + 43; }", 42);

exits!(float_arithmetic, "int main(void) { double d; d = 21.0; return d * 2; }", 42);
exits!(float_local, "int main(void) { float f; f = 10.5; return f * 4; }", 42);
exits!(double_from_integer_literal, "int main(void) { double d; d = 21; return d * 2; }", 42);
exits!(double_inexact_literal, "int main(void) { double d; d = 0.1; return d * 420; }", 42);
exits!(double_zero_initializer, "int main(void) { double d = 0; return d + 42; }", 42);
exits!(negative_zero, "int main(void) { double d; d = 0.0; d = -d; return 1.0 / d < 0 ? 42 : 7; }", 42);

exits!(call_void, "void f(void) { return; } int main(void) { f(); return 42; }", 42);
exits!(call_argument, "int f(int x) { return x + 1; } int main(void) { return f(41); }", 42);

exits!(initializer_evaluated_once, "int main(void) { int x = 40; int y = x++; return x + y; }", 81);
