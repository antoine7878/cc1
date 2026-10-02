#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(use_plain_read, "int x = 41; int f(void) { return x; } int main(void) { return f(); }", 41, "");
valid!(use_sizeof_ident_not_used, "extern int x; int f(void) { return sizeof x; } int main(void) { return f(); }", 4, "");
valid!(use_sizeof_expr_not_used, "extern int x; int f(void) { return sizeof(x + 1); } int main(void) { return f(); }", 4, "");
valid!(use_sizeof_operand_precedence, "extern int x; int y = 2; int f(void) { return sizeof x + y; } int main(void) { return f(); }", 6, "");
valid!(use_function_call, "int f(void); int g(void) { return f(); } int f(void) { return 17; } int main(void) { return g(); }", 17, "");
valid!(use_address_of, "int x = 7; int *p; void f(void) { p = &x; } int main(void) { f(); return *p; }", 7, "");
valid!(use_member_access_marks_struct_var, "struct S { int m; }; struct S s = { 9 }; int f(void) { return s.m; } int main(void) { return f(); }", 9, "");
valid!(use_never_referenced, "extern int x; int main(void) { return 0; }", 0, "");
valid!(use_shadowed_by_inner_scope, "int x = 1; int f(void) { int x = 3; return x; } int main(void) { return f() + x; }", 4, "");
