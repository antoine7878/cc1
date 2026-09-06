// 6.5.7 Initialization

accept!(init_struct_list, "struct S { int a; int b; }; struct S s = { 1, 2 };");
accept!(init_struct_partial, "struct S { int a; int b; }; struct S s = { 1 };");
accept!(
    init_struct_brace_elision,
    "struct S { int a; int b[2]; int c; }; struct S s = { 1, 2, 3, 4 };"
);
accept!(
    init_struct_fully_braced,
    "struct S { int a; int b[2]; int c; }; struct S s = { 1, { 2, 3 }, 4 };"
);
accept!(
    init_struct_nested,
    "struct I { int x; }; struct S { struct I i; }; struct S s = { { 1 } };"
);
accept!(
    init_union_first_member,
    "union U { int a; double b; }; union U u = { 1 };"
);
accept!(
    init_array_of_struct,
    "struct S { int a; int b; }; struct S s[2] = { { 1, 2 }, { 3, 4 } };"
);
accept!(
    init_unnamed_bitfield_skipped,
    "struct S { int a : 3; int : 2; int b : 3; }; struct S s = { 1, 2 };"
);

reject!(init_struct_too_many, "struct S { int a; }; struct S s = { 1, 2 };");
reject!(
    init_union_too_many,
    "union U { int a; double b; }; union U u = { 1, 2 };"
);
reject!(
    init_nested_inner_too_many,
    "struct S { int a; int b[2]; }; struct S s = { 1, { 2, 3, 4 } };"
);

accept!(init_char_array_from_string, "char s[4] = \"abc\";");
accept!(init_char_array_exact_drops_nul, "char s[3] = \"abc\";");
accept!(init_char_array_braced_string, "char s[4] = { \"abc\" };");
accept!(init_char_pointer_from_string, "char *p = \"abc\";");
reject!(init_char_array_string_too_long, "char s[2] = \"abc\";");
reject!(init_int_array_from_string, "int a[4] = \"abc\";");

size!(init_char_array_sized_from_string, "char a[] = \"abc\";", "a", 4);
size!(init_array_sized_from_list, "int a[] = { 1, 2, 3 };", "a", 12);
size!(
    init_array_of_struct_sized_from_list,
    "struct S { int a; int b; }; struct S a[] = { { 1, 2 }, { 3, 4 } };",
    "a",
    16
);

// 6.5.7: expressions in an initializer for a static-duration object, or in an
// initializer list for an aggregate or union, shall be constant expressions.
accept!(
    init_automatic_scalar_from_variable,
    "int g; void f(void) { int x = g; }"
);
accept!(init_static_scalar_constant, "void f(void) { static int x = 1 + 2; }");
accept!(init_address_constant, "int g; int *p = &g;");
accept!(init_array_decay_constant, "int a[3]; int *p = a;");
accept!(init_function_address_constant, "int f(void); int (*p)(void) = f;");

reject!(init_file_scope_from_variable, "int g; int x = g;");
reject!(
    init_block_static_from_variable,
    "int g; void f(void) { static int x = g; }"
);
reject!(
    init_automatic_aggregate_from_variable,
    "struct S { int a; int b; }; void f(int n) { struct S s = { n, 1 }; }"
);
reject!(
    init_automatic_array_from_variable,
    "void f(int n) { int a[2] = { n, 1 }; }"
);
