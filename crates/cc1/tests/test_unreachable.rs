#![cfg_attr(rustfmt, rustfmt_skip)]

exits!(dead_or_after_return, "int f(int x) { return x; x = x || 1; } int main(void) { return f(3); }", 3);
exits!(dead_and_after_break_before_case, "int f(int a, int b) { switch (a) { case 1: break; b = a && b; case 2: return b; } return 0; } int main(void) { return f(2, 5); }", 5);
exits!(dead_and_before_first_case, "int f(int x) { switch (x) { x = x && 1; case 1: return x; } return 0; } int main(void) { return f(1); }", 1);
exits!(dead_and_after_goto, "int f(int x) { goto out; x = x && 2; out: return x; } int main(void) { return f(4); }", 4);
exits!(dead_logical_after_continue, "int f(int x) { int i; for (i = 0; i < 2; i++) { continue; x = (x && i) || x; } return x; } int main(void) { return f(5); }", 5);
exits!(dead_conditional_store_in_void_function, "void f(int a, int *r) { return; *r = a ? 1 : 2; } int main(void) { int r = 9; f(1, &r); return r; }", 9);
exits!(dead_conditional_in_call_argument, "int g(int x, int y) { return x + y; } int f(int a) { return 0; return g(a + 1, a ? 1 : 2); } int main(void) { return f(1); }", 0);
exits!(dead_if_with_label_reached_by_goto, "int f(int a, int b) { goto in; if (a && b) { in: return 7; } return 1; } int main(void) { return f(0, 0); }", 7);
exits!(dead_compound_initializer_with_label_reached_by_goto, "int f(int a) { goto in; { int x = a && 1; in: return 7; } } int main(void) { return f(0); }", 7);
exits!(dead_while_condition_with_label_reached_by_goto, "int f(int a) { goto in; while (a || 1) { in: return 7; } return 1; } int main(void) { return f(0); }", 7);
exits!(case_inside_loop_body_duff_style, "int f(int n) { int s = 0; switch (n % 2) { case 0: while (n > 0) { s++; n--; case 1: s++; n--; } } return s; } int main(void) { return f(3); }", 3);
exits!(case_inside_dead_loop_before_first_case, "int f(int n) { int s = 0; switch (n) { s = 9; while (0) { case 1: s++; } } return s; } int main(void) { return f(1); }", 1);
emits!(not dead_switch_without_labels_is_not_emitted, "int f(int i) { return 0; switch (i) { case 1: i = 2; } }", "switch i32");
emits!(not dead_call_is_not_emitted, "int g(void); int f(void) { return 0; g(); }", "call");
