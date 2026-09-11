// 6.5.7 Initialization

accept!(init_struct_list, "struct S { int a; int b; }; struct S s = { 1, 2 };");
accept!(init_struct_partial, "struct S { int a; int b; }; struct S s = { 1 };");
accept!(init_struct_brace_elision, "struct S { int a; int b[2]; int c; }; struct S s = { 1, 2, 3, 4 };");
accept!(init_struct_fully_braced, "struct S { int a; int b[2]; int c; }; struct S s = { 1, { 2, 3 }, 4 };");
accept!(init_struct_nested, "struct I { int x; }; struct S { struct I i; }; struct S s = { { 1 } };");
accept!(init_union_first_member, "union U { int a; double b; }; union U u = { 1 };");
accept!(init_array_of_struct, "struct S { int a; int b; }; struct S s[2] = { { 1, 2 }, { 3, 4 } };");
accept!(init_unnamed_bitfield_skipped, "struct S { int a : 3; int : 2; int b : 3; }; struct S s = { 1, 2 };");

reject!(init_struct_too_many, "struct S { int a; }; struct S s = { 1, 2 };");
reject!(init_union_too_many, "union U { int a; double b; }; union U u = { 1, 2 };");
reject!(init_nested_inner_too_many, "struct S { int a; int b[2]; }; struct S s = { 1, { 2, 3, 4 } };");

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
accept!(init_automatic_scalar_from_variable, "int g; void f(void) { int x = g; }");
accept!(init_static_scalar_constant, "void f(void) { static int x = 1 + 2; }");
accept!(init_address_constant, "int g; int *p = &g;");
accept!(init_array_decay_constant, "int a[3]; int *p = a;");
accept!(init_function_address_constant, "int f(void); int (*p)(void) = f;");

reject!(init_file_scope_from_variable, "int g; int x = g;");
reject!(init_block_static_from_variable, "int g; void f(void) { static int x = g; }");
reject!(init_automatic_aggregate_from_variable, "struct S { int a; int b; }; void f(int n) { struct S s = { n, 1 }; }");
reject!(init_automatic_array_from_variable, "void f(int n) { int a[2] = { n, 1 }; }");

// 6.4: an address constant may be built with &, *, [], . and pointer casts,
// but the value of an object shall not be accessed by those operators.
inits!(init_folds_address_of_object, "int g; int *p = &g;", &[("p", "&g")]);
inits!(init_folds_array_decay, "int a[3]; int *p = a;", &[("p", "&a")]);
inits!(init_folds_function_designator, "int f(void); int (*p)(void) = f;", &[("p", "&f")]);
inits!(init_folds_address_of_element, "int a[10]; int *p = &a[3];", &[("p", "&a+12")]);
inits!(init_folds_address_of_member, "struct S { int x; int y; } s; int *p = &s.y;", &[("p", "&s+4")]);
inits!(
    init_folds_address_of_element_of_member,
    "struct S { int x; int a[4]; } s; int *p = &s.a[2];",
    &[("p", "&s+12")]
);
inits!(init_folds_member_array_decay, "struct S { int x; int a[4]; } s; int *p = s.a;", &[("p", "&s+4")]);
inits!(init_folds_address_of_row_element, "int a[3][4]; int *p = &a[1][2];", &[("p", "&a+24")]);
inits!(init_folds_pointer_addition, "int a[10]; int *p = a + 3;", &[("p", "&a+12")]);
inits!(init_folds_pointer_subtraction, "int a[10]; int *p = &a[5] - 2;", &[("p", "&a+12")]);
inits!(init_folds_to_a_negative_offset, "int a[10]; int *p = &a[0] - 2;", &[("p", "&a-8")]);
inits!(init_folds_byte_offset_through_a_cast, "int x; char *p = (char *)&x + 1;", &[("p", "&x+1")]);
inits!(init_folds_offset_into_a_string_literal, "char *p = \"abc\" + 1;", &[("p", "&\"abc\"+1")]);
inits!(init_folds_indirection_of_address, "int x; int *p = &*&x;", &[("p", "&x")]);
inits!(init_folds_absolute_address, "int *p = (int *)4 + 1;", &[("p", "&abs+8")]);
inits!(init_folds_address_constant_into_an_integer, "int a[10]; long n = (long)&a[3];", &[("n", "&a+12")]);

reject!(init_address_read_through_a_pointer_variable, "struct S { int x; } s; struct S *q = &s; int *p = &q->x;");
reject!(init_address_of_an_automatic_object, "void f(void) { int x; static int *p = &x; }");
reject!(init_static_struct_from_a_variable, "struct S { int a; } x; struct S y = x;");
inits!(init_long_double_constant, "long double x = 3.3L;", &[("x", "LongDouble(3.3)")]);
