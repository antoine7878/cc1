#![cfg_attr(rustfmt, rustfmt_skip)]

exits!(control_if_array_decays, "int a[3]; int main(void) { if (a) return 1; return 0; }", 1);
exits!(control_while_function_decays, "int main(void) { while (main) return 2; return 0; }", 2);
exits!(control_for_string_decays, "int main(void) { for (; \"x\"; ) return 3; return 0; }", 3);
exits!(control_do_array_decays, "int a[3]; int main(void) { do { return 4; } while (a); }", 4);
exits!(control_not_array_is_false, "int a[3]; int main(void) { int n = 0; while (!a) n = 7; return n; }", 0);

exits!(typedef_restored_after_parameter_shadow, "typedef int T; int f(int T) { return T; } T g = 3; int main(void) { return f(1) + g; }", 4);
exits!(typedef_restored_after_local_shadow, "typedef int T; int f(void) { int T = 2; return T; } T g = 3; int main(void) { return f() + g; }", 5);
exits!(typedef_inner_block_and_outer_variable, "typedef int T; int main(void) { int T; { typedef int T; T y; y = 0; } T = 1; return T; }", 1);
exits!(typedef_param_scope_of_function_returning_pointer, "typedef int T; int (*f(int T))(int b) { int x = (T); return 0; } int main(void) { return f(1) == 0; }", 1);
exits!(typedef_enumerator_shadows_in_block, "typedef int T; int main(void) { enum { T = 5 }; return (T); }", 5);
exits!(warns typedef_enumerator_shadows_in_parameter, "typedef int T; int f(enum { T } e) { return (T) + e; } int main(void) { return f(1); }", 1);
exits!(typedef_enumerator_in_typedef_is_ordinary, "typedef int T; int main(void) { typedef enum { A, T } E; return (T); }", 1);
exits!(typedef_enumerator_in_struct_is_ordinary, "typedef int T; int main(void) { struct S { enum { A, T } e; } s; s.e = A; return (T); }", 1);
exits!(typedef_abstract_function_parameters_do_not_leak, "typedef int T; void f(int (int T), T y); int main(void) { return 0; }", 0);

exits!(parameter_visible_to_following_parameter, "int f(int n, char (*p)[sizeof(n)]) { return (int)sizeof(*p); } int main(void) { char a[4]; return f(1, &a); }", 4);

exits!(
    null_void_pointer_in_parentheses_for_function_pointers,
    "int f(void) { return 1; } int (*fp)(void) = ((void *)0); int (*g(void))(void) { return ((void *)0); } int call(int (*h)(void)) { return h == 0; } int main(void) { int (*q)(void); q = ((void *)0); q = 1 ? f : ((void *)0); return (fp == ((void *)0)) + call(((void *)0)) + (g() == 0) + q(); }",
    4
);
exits!(null_parenthesized_zero_in_cast, "int *p = (void *)(0); int main(void) { return p == 0; }", 1);
accept!(null_enum_cast_is_null_pointer_constant, "enum E { Z }; int *p = (enum E)0; int main(void) { return p != 0; }");

exits!(
    static_address_parenthesized_operands,
    "int x, arr[4]; struct S { int a; int b[3]; } s; int *p1 = &(x); int *p2 = (arr) + 1; char *p3 = (\"abc\"); int *p4 = &(arr[2]); int *p5 = &(&s)->a; int main(void) { return (p1 == &x) + (p2 == arr + 1) + (*p3 == 'a') + (p4 == &arr[2]) + (p5 == &s.a); }",
    5
);
exits!(static_offsetof_idiom, "struct S { int a; int b[3]; }; int off = (int)&((struct S *)0)->b[1]; int main(void) { return off; }", 8);

exits!(
    static_float_division_by_zero,
    "double inf = 1.0 / 0.0; double ninf = -1.0 / 0.0; double nan = 0.0 / 0.0; int main(void) { return (inf > 1e308) + (ninf < 0) * 2 + (nan != nan) * 4; }",
    7
);
reject!(static_integer_division_by_zero, "int x = 1 / 0;");
reject!(static_integer_modulo_by_zero, "int x = 1 % 0;");

exits!(warns void_parameter_named_in_declaration, "int f(int a, void x); int main(void) { return 0; }", 0);
reject!(void_parameter_unnamed_not_alone_last, "int f(int, void); int main(void) { return 0; }");
reject!(void_parameter_unnamed_not_alone_first, "int f(void, int); int main(void) { return 0; }");
reject!(void_parameter_qualified_alone, "int f(const void); int main(void) { return 0; }");
reject!(void_parameter_named_in_definition, "int f(void x) { return 0; } int main(void) { return 0; }");
