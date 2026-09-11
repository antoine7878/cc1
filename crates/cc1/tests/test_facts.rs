// Every fact a code generator reads must be present for a program with no diagnostics.

facts!(facts_empty_function, "void f(void){ }");
facts!(facts_locals_and_arithmetic, "int f(int a, int b){ int c = a + b * 2; return c; }");
facts!(facts_pointers, "void f(int *p){ int x = *p; p = &x; p[0] = x; }");
facts!(facts_members, "struct S { int a; struct S *next; }; int f(struct S *s){ return s->a + s->next->a; }");
facts!(facts_bitfields, "struct S { unsigned a : 3; int b : 5; }; int f(struct S *s){ return s->a + s->b; }");
facts!(facts_union, "union U { int a; double b; }; double f(union U *u){ u->a = 1; return u->b; }");
facts!(facts_arrays, "int f(void){ int a[3][4]; a[1][2] = 5; return a[1][2]; }");
facts!(
    facts_loops,
    "int f(int n){ int i, s = 0; for (i = 0; i < n; i++) { if (i == 3) continue; s += i; if (s > 9) break; } return s; }"
);
facts!(facts_while_and_do, "int f(int n){ while (n) { n--; } do { n++; } while (n < 3); return n; }");
facts!(facts_switch, "int f(int x){ switch (x) { case 1: return 1; case 2: break; default: return 0; } return x; }");
facts!(
    facts_switch_in_loop,
    "int f(int x){ while (x) { switch (x) { case 1: continue; default: break; } x--; } return x; }"
);
facts!(facts_goto, "int f(int x){ if (x) goto out; x = 1; out: return x; }");
facts!(facts_strings, "char *f(void){ char *p = \"abc\"; return p + 1; }");
facts!(facts_calls, "int g(int, double); int f(void){ return g(1, 2.0); }");
facts!(facts_variadic_call, "int g(int, ...); int f(void){ return g(1, 2, 3.0, \"s\"); }");
facts!(facts_casts, "int f(double d){ return (int)d + (char)1; }");
facts!(facts_conditional, "int f(int a){ return a ? a + 1 : -a; }");
facts!(facts_enum, "enum E { A, B = 4 }; int f(enum E e){ return e == A ? B : e; }");
facts!(facts_function_pointer, "int g(void); int f(void){ int (*p)(void) = g; return p(); }");
facts!(facts_statics, "static int counter; int f(void){ static int local = 2; counter += local; return counter; }");
facts!(facts_sizeof, "struct S { char c[7]; }; int f(void){ return (int)(sizeof(struct S) + sizeof(int)); }");
facts!(facts_compound_assign, "int f(int a){ double d = 1.5; a += 2; a <<= 1; d *= a; return a + (int)d; }");
facts!(facts_comma_and_nested_blocks, "int f(void){ int x = 0; { int y = (x = 1, x + 1); { x = y; } } return x; }");
facts!(
    facts_global_initializers,
    "int a[3] = { 1, 2, 3 }; char s[] = \"hi\"; int *p = &a[1]; struct S { int x, y; } g = { 1, 2 };"
);
