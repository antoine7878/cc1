#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(an_inner_declaration_shadows_an_outer_one, "int x = 7; int main(void) { int result = x; { int x = 3; result += x; } return result + x; }", 17, "");
valid!(ordinary_and_tag_namespaces_are_separate, "struct S { int S; }; int S = 3; int main(void) { struct S s; s.S = 4; return S + s.S; }", 7, "");
valid!(prototype_scope_does_not_leak, "int f(int x); int x = 9; int f(int a) { return a + x; } int main(void) { return f(3); }", 12, "");
valid!(labels_have_function_scope, "int f(int n) { if (n) goto out; return 1; out: return 2; } int g(void) { goto out; out: return 3; } int main(void) { return f(1) + g(); }", 5, "");
