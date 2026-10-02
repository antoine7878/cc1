#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(facts_empty_function, "void f(void){ }\nint main(void) { f(); return 0; }", 0, "");
valid!(facts_locals_and_arithmetic, "int f(int a, int b){ int c = a + b * 2; return c; }\nint main(void) { return f(3, 4); }", 11, "");
valid!(facts_pointers, "void f(int *p){ int x = *p; p = &x; p[0] = x; }\nint main(void) { int value = 7; f(&value); return value; }", 7, "");
valid!(facts_members, "struct S { int a; struct S *next; }; int f(struct S *s){ return s->a + s->next->a; }\nint main(void) { struct S next = { 2, 0 }; struct S value = { 0 }; value.a = 1; value.next = &next; return f(&value); }", 3, "");
valid!(facts_bitfields, "struct S { unsigned a : 3; int b : 5; }; int f(struct S *s){ return s->a + s->b; }\nint main(void) { struct S value = { 1, 2 }; return f(&value); }", 3, "");
valid!(facts_union, "union U { int a; double b; }; double f(union U *u){ u->a = 1; return u->b; }\nint main(void) { return 0; }", 0, "");
valid!(facts_arrays, "int f(void){ int a[3][4]; a[1][2] = 5; return a[1][2]; }\nint main(void) { return f(); }", 5, "");
valid!(facts_loops, "int f(int n){ int i, s = 0; for (i = 0; i < n; i++) { if (i == 3) continue; s += i; if (s > 9) break; } return s; }\nint main(void) { return f(8); }", 12, "");
valid!(facts_while_and_do, "int f(int n){ while (n) { n--; } do { n++; } while (n < 3); return n; }\nint main(void) { return f(5); }", 3, "");
valid!(facts_switch, "int f(int x){ switch (x) { case 1: return 1; case 2: break; default: return 0; } return x; }\nint main(void) { return f(1) + f(2) + f(0); }", 3, "");
valid!(facts_switch_in_loop, "int f(int x){ while (x) { switch (x) { case 1: continue; default: break; } x--; } return x; }\nint main(void) { return 0; }", 0, "");
valid!(facts_goto, "int f(int x){ if (x) goto out; x = 1; out: return x; }\nint main(void) { return f(0) + f(4); }", 5, "");
valid!(facts_strings, "char *f(void){ char *p = \"abc\"; return p + 1; }\nint main(void) { char *p = f(); return !(p[0] == 98 && p[1] == 99 && p[2] == 0); }", 0, "");
valid!(facts_calls, "int g(int, double); int f(void){ return g(1, 2.0); }\nint g(int a, double b) { return a + (int)b; }\nint main(void) { return f(); }", 3, "");
valid!(facts_variadic_call, "int g(int, ...); int f(void){ return g(1, 2, 3.0, \"s\"); }\nint main(void) { return f(); }", 7, "", helper = "int g(int first, ...) { return first + 6; }");
valid!(facts_casts, "int f(double d){ return (int)d + (char)1; }\nint main(void) { return f(2.5); }", 3, "");
valid!(facts_conditional, "int f(int a){ return a ? a + 1 : -a; }\nint main(void) { return f(0) + f(5) + f(-3); }", 4, "");
valid!(facts_enum, "enum E { A, B = 4 }; int f(enum E e){ return e == A ? B : e; }\nint main(void) { return f(A) + f(B); }", 8, "");
valid!(facts_function_pointer, "int g(void); int f(void){ int (*p)(void) = g; return p(); }\nint g(void) { return 17; }\nint main(void) { return f(); }", 17, "");
valid!(facts_statics, "static int counter; int f(void){ static int local = 2; counter += local; return counter; }\nint main(void) { int a = f(); int b = f(); return a + b; }", 6, "");
valid!(facts_sizeof, "struct S { char c[7]; }; int f(void){ return (int)(sizeof(struct S) + sizeof(int)); }\nint main(void) { return f(); }", 11, "");
valid!(facts_compound_assign, "int f(int a){ double d = 1.5; a += 2; a <<= 1; d *= a; return a + (int)d; }\nint main(void) { return f(3); }", 25, "");
valid!(facts_comma_and_nested_blocks, "int f(void){ int x = 0; { int y = (x = 1, x + 1); { x = y; } } return x; }\nint main(void) { return f(); }", 2, "");
valid!(facts_global_initializers, "int a[3] = { 1, 2, 3 }; char s[] = \"hi\"; int *p = &a[1]; struct S { int x, y; } g = { 1, 2 };\nint main(void) { return !(a[0] == 1 && a[2] == 3 && *p == 2 && s[0] == 104 && s[1] == 105 && g.x == 1 && g.y == 2); }", 0, "");
