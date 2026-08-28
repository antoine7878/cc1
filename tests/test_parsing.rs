mod common;

// ---- 6.5.6 typedef and lexerhack -----------------------

accept!(typedef_then_use, "typedef int T; T x;");

accept!(typedef_use_in_block, "typedef int T; void f(void) { T x; x = 1; }");

accept!(typedef_chain, "typedef int A; typedef A B; B x;");

accept!(typedef_of_struct_tag, "struct S { int x; }; typedef struct S S; S v;");

accept!(forward_struct_typedef, "typedef struct S *SP; struct S { SP p; };");

accept!(cast_to_typedef, "typedef int T; void f(void) { int x; x = (T)1; }");

accept!(
    typedef_redef_in_block,
    "typedef int T; void f(void) { typedef T T; T x; x = 1; }"
);

accept!(
    knr_definition_with_typedef_param_type,
    "typedef int T; f(a) T a; { return a; }"
);

accept!(block_typedef_no_leak, "void f(void) { typedef int T; } int T;");

reject!(
    nested_block_typedef_no_leak,
    "typedef int T; void f(void) { if (1) { typedef char U; } } U u;"
);

reject!(enumerator_conflicts_with_typedef, "typedef int T; enum E { T };");

reject!(knr_param_named_as_typedef, "typedef int a; f(a) int a; { return a; }");

accept!(
    sizeof_shadowed_verdict,
    "typedef int T; int main(void) { int T; return sizeof(T); }"
);

accept!(
    local_var_shadows_typedef,
    "typedef int T; int main(void) { int T; T = 1; return T; }"
);

accept!(param_shadows_typedef, "typedef int T; T f(T T) { return T; }");

accept!(
    paren_expr_of_shadowed_typedef,
    "typedef int T; int main(void) { int T; return (T); }"
);

accept!(
    sizeof_expr_on_shadowed_typedef,
    "typedef int T; int main(void) { int T; return sizeof T; }"
);

accept!(label_named_as_typedef, "typedef int T; void f(void) { T: ; goto T; }");

accept!(struct_tag_named_as_typedef, "typedef int S; struct S { int x; };");

accept!(union_tag_named_as_typedef, "typedef int S; union S { int x; };");

accept!(member_named_as_typedef, "typedef int T; struct S { T T; };");

accept!(
    member_access_named_as_typedef,
    "typedef int x; struct S { int x; }; void f(void) { struct S s; s.x = 1; }"
);

reject!(typedef_redeclared_as_var, "typedef int T; int T;");

accept!(
    knr_struct_def_in_declaration_list,
    "f(a) struct S { int x; } *a; { return 0; }"
);

accept!(member_shadows_outer_typedef, "typedef int T; struct S { int T; }; T x;");

accept!(enum_tag_named_as_typedef, "typedef int E; enum E { A };");

accept!(struct_body_then_typedef_decl, "typedef struct { int x; } S; S s;");

accept!(multiword_type_then_typedef, "typedef unsigned long UL; UL u;");

accept!(
    typedef_after_nontypedef_decl,
    "typedef int T; int y; void f(void) { y = 1; }"
);

reject!(pointer_declarator_typedef, "typedef int T; int *T;");

// ---- 6.3.1 primary expressions ------------------------------------------

syntax!(primary_identifier, "int x; int f(void) { return x; }");
syntax!(primary_parenthesized, "int f(void) { return (((1))); }");
syntax!(constant_decimal, "int f(void) { return 42; }");
syntax!(constant_octal, "int f(void) { return 0777; }");
syntax!(constant_hex, "int f(void) { return 0xFFu; }");
syntax!(
    constant_suffixes,
    "unsigned long f(void) { return 1ul + 2UL + 3L + 4U; }"
);
syntax!(constant_char, "int f(void) { return 'a'; }");
syntax!(
    constant_char_escape,
    "int f(void) { return '\\n' + '\\0' + '\\\\' + '\\''; }"
);
syntax!(constant_wide_char, "int f(void) { return L'a'; }");
syntax!(
    constant_float,
    "double f(void) { return 1.0 + .5 + 2. + 1e10 + 1.5e-3 + 1.0f + 1.0L; }"
);
syntax!(string_literal_expr, "char *f(void) { return \"hello\"; }");
syntax!(string_literal_escapes, "char *f(void) { return \"a\\tb\\\"c\\\\d\"; }");
syntax!(string_literal_wide, "int f(void) { return L\"wide\"[0]; }");

// ---- 6.3.2 postfix expressions ------------------------------------------

syntax!(postfix_subscript, "int a[4]; int f(void) { return a[2]; }");
syntax!(postfix_subscript_nested, "int a[4][5]; int f(void) { return a[1][2]; }");
syntax!(postfix_call_no_args, "int g(void); int f(void) { return g(); }");
syntax!(postfix_call_one_arg, "int g(int); int f(void) { return g(1); }");
syntax!(
    postfix_call_many_args,
    "int g(int, int, int); int f(void) { return g(1, 2, 3); }"
);
syntax!(postfix_call_comma_arg, "int g(int); int f(void) { return g((1, 2)); }");
syntax!(
    postfix_call_through_pointer,
    "int (*fp)(int); int f(void) { return (*fp)(1) + fp(2); }"
);
syntax!(postfix_member_dot, "struct S { int m; } s; int f(void) { return s.m; }");
syntax!(
    postfix_member_arrow,
    "struct S { int m; } *p; int f(void) { return p->m; }"
);
syntax!(
    postfix_member_chain,
    "struct A { int m; }; struct B { struct A a; } b; int f(void) { return b.a.m; }"
);
syntax!(postfix_inc_dec, "int f(void) { int x; x = 0; x++; x--; return x; }");

// ---- 6.3.3 unary expressions --------------------------------------------

syntax!(
    unary_prefix_inc_dec,
    "int f(void) { int x; x = 0; ++x; --x; return x; }"
);
syntax!(
    unary_address_and_deref,
    "int f(void) { int x; int *p; x = 0; p = &x; return *p; }"
);
syntax!(
    unary_arithmetic,
    "int f(void) { int x; x = 1; return +x + -x + ~x + !x; }"
);
syntax!(unary_sizeof_expr, "int f(void) { int x; x = 0; return sizeof x; }");
syntax!(
    unary_sizeof_paren_expr,
    "int f(void) { int x; x = 0; return sizeof (x); }"
);
syntax!(unary_sizeof_type, "int f(void) { return sizeof(int); }");
syntax!(
    unary_sizeof_ptr_type,
    "int f(void) { return sizeof(char *) + sizeof(int (*)(void)); }"
);
syntax!(unary_sizeof_array_type, "int f(void) { return sizeof(int[10]); }");

// ---- 6.3.4 cast expressions ---------------------------------------------

syntax!(cast_scalar, "int f(void) { return (int)1.5; }");
syntax!(cast_pointer, "int f(void) { int x; x = 0; return *(int *)&x; }");
syntax!(cast_void, "void g(void); void f(void) { (void)0; }");
syntax!(cast_nested, "int f(void) { return (int)(char)(long)1; }");
syntax!(
    cast_to_function_pointer,
    "void f(void) { int (*p)(int); p = 0; (void)(int (*)(int))p; }"
);

// ---- 6.3.5 - 6.3.14 binary operators ------------------------------------

syntax!(binary_multiplicative, "int f(int a, int b) { return a * b / a % b; }");
syntax!(binary_additive, "int f(int a, int b) { return a + b - a; }");
syntax!(binary_shift, "int f(int a, int b) { return a << b >> a; }");
syntax!(
    binary_relational,
    "int f(int a, int b) { return (a < b) + (a > b) + (a <= b) + (a >= b); }"
);
syntax!(binary_equality, "int f(int a, int b) { return (a == b) + (a != b); }");
syntax!(binary_bitwise, "int f(int a, int b) { return (a & b) | (a ^ b); }");
syntax!(binary_logical, "int f(int a, int b) { return (a && b) || (a || b); }");
syntax!(
    binary_precedence_mix,
    "int f(int a, int b, int c) { return a + b * c - a / b % c; }"
);

// ---- 6.3.15 - 6.3.17 conditional, assignment, comma ---------------------

syntax!(conditional_expr, "int f(int a) { return a ? 1 : 2; }");
syntax!(conditional_nested, "int f(int a) { return a ? a ? 1 : 2 : 3; }");
syntax!(assignment_simple, "int f(void) { int x; x = 1; return x; }");
syntax!(
    assignment_compound,
    "int f(void) { int x; x = 1; x += 1; x -= 1; x *= 2; x /= 2; x %= 2; x <<= 1; x >>= 1; x &= 1; x ^= 1; x |= 1; return x; }"
);
syntax!(assignment_chained, "int f(void) { int a, b; a = b = 1; return a; }");
syntax!(comma_expr, "int f(void) { int x; x = (1, 2, 3); return x; }");

// ---- 6.5.1 storage-class specifiers -------------------------------------

syntax!(storage_extern, "extern int x;");
syntax!(storage_static_file, "static int x; static int f(void) { return x; }");
syntax!(
    storage_auto_register,
    "int f(void) { auto int x; register int y; x = 0; y = 0; return x + y; }"
);
syntax!(storage_typedef, "typedef int T;");
syntax!(storage_before_and_after_type, "static int a; int static b;");

// ---- 6.5.2 type specifiers ----------------------------------------------

syntax!(type_void_function, "void f(void) { return; }");
syntax!(type_plain_char, "char c;");
syntax!(type_signed_unsigned_char, "signed char a; unsigned char b;");
syntax!(
    type_short_forms,
    "short a; short int b; signed short c; signed short int d; unsigned short e; unsigned short int f;"
);
syntax!(
    type_int_forms,
    "int a; signed b; signed int c; unsigned d; unsigned int e;"
);
syntax!(
    type_long_forms,
    "long a; long int b; signed long c; signed long int d; unsigned long e; unsigned long int f;"
);
syntax!(type_float_double, "float a; double b; long double c;");
syntax!(
    type_specifier_order_swapped,
    "int unsigned a; int long b; char signed c;"
);

// ---- 6.5.2.1 struct and union specifiers --------------------------------

syntax!(struct_with_tag_and_body, "struct S { int a; char b; };");
syntax!(struct_anonymous_body, "struct { int a; } s;");
syntax!(struct_tag_only_reference, "struct S { int a; }; struct S s;");
syntax!(struct_forward_reference, "struct S; struct S *p;");
syntax!(struct_self_referential, "struct Node { int v; struct Node *next; };");
syntax!(
    struct_mutually_recursive,
    "struct A { struct B *b; }; struct B { struct A *a; };"
);
syntax!(
    struct_nested_definition,
    "struct Outer { struct Inner { int x; } in; };"
);
syntax!(struct_multiple_declarators, "struct S { int a, b, *c, d[4]; };");
syntax!(
    struct_bitfields,
    "struct S { unsigned a : 1; signed b : 3; int c : 2; };"
);
syntax!(
    struct_anonymous_bitfield,
    "struct S { unsigned a : 1; unsigned : 2; int b; };"
);
syntax!(
    struct_zero_width_bitfield,
    "struct S { unsigned a : 1; unsigned : 0; unsigned b : 1; };"
);
syntax!(
    struct_qualified_members,
    "struct S { const int a; volatile int b; const volatile int c; };"
);
syntax!(union_with_tag_and_body, "union U { int i; float f; char c[4]; };");
syntax!(union_anonymous_body, "union { int i; float f; } u;");
syntax!(union_nested_in_struct, "struct S { union { int i; float f; } u; };");

// ---- 6.5.2.2 enumeration specifiers -------------------------------------

syntax!(enum_with_tag, "enum E { A, B, C };");
syntax!(enum_anonymous, "enum { A, B }; ");
syntax!(enum_explicit_values, "enum E { A = 1, B = 2, C = A + B };");
syntax!(enum_mixed_values, "enum E { A, B = 5, C };");
syntax!(enum_single_enumerator, "enum E { A };");
syntax!(enum_tag_reference, "enum E { A }; enum E e;");
syntax!(enum_variable_declaration, "enum E { A } e;");

// ---- 6.5.3 type qualifiers ----------------------------------------------

syntax!(qualifier_const, "const int a = 1;");
syntax!(qualifier_volatile, "volatile int a;");
syntax!(qualifier_both_orders, "const volatile int a; volatile const int b;");
syntax!(qualifier_after_type, "int const a; int volatile b;");
syntax!(
    qualifier_on_pointer,
    "int *const p = 0; const int *q; const int *const r = 0;"
);
syntax!(qualifier_with_storage, "static const int a = 1; extern volatile int b;");

// ---- 6.5.4 declarators ---------------------------------------------------

syntax!(declarator_pointer_levels, "int *a; int **b; int ***c;");
syntax!(declarator_qualified_pointers, "int *const *volatile p;");
syntax!(declarator_parenthesized, "int (a); int (*(b));");
syntax!(declarator_array_sized, "int a[10];");
syntax!(declarator_array_unsized_extern, "extern int a[];");
syntax!(declarator_array_multidim, "int a[2][3][4];");
syntax!(declarator_array_of_pointers, "int *a[10];");
syntax!(declarator_pointer_to_array, "int (*a)[10];");
syntax!(declarator_array_const_expr_size, "int a[2 + 3 * 4];");
syntax!(declarator_function_prototype, "int f(int a, char b);");
syntax!(declarator_function_unnamed_params, "int f(int, char);");
syntax!(declarator_function_void_params, "int f(void);");
syntax!(declarator_function_empty_params, "int f();");
syntax!(declarator_function_variadic, "int f(int a, ...);");
syntax!(declarator_function_pointer, "int (*fp)(int);");
syntax!(declarator_array_of_function_pointers, "int (*fp[4])(int);");
syntax!(declarator_function_returning_pointer, "int *f(void);");
syntax!(declarator_function_returning_function_pointer, "int (*f(int))(int);");
syntax!(declarator_nested_pointer_array_mix, "int *(*(*a)[4])(int);");
syntax!(declarator_param_array_adjusts, "int f(int a[], int b[10], int c[][4]);");
syntax!(declarator_param_function_pointer, "int f(int (*cb)(int));");
syntax!(declarator_param_qualified, "int f(const int *p, int *const q);");

// ---- 6.5.7 initialization ------------------------------------------------

syntax!(init_scalar, "int a = 1;");
syntax!(init_scalar_expr, "int a = 1 + 2 * 3;");
syntax!(init_multiple_declarators, "int a = 1, b = 2, c;");
syntax!(init_array_braced, "int a[3] = { 1, 2, 3 };");
syntax!(init_array_trailing_comma, "int a[3] = { 1, 2, 3, };");
syntax!(init_array_partial, "int a[5] = { 1, 2 };");
syntax!(init_array_inferred_size, "int a[] = { 1, 2, 3 };");
syntax!(init_array_nested_braces, "int a[2][2] = { { 1, 2 }, { 3, 4 } };");
syntax!(init_array_flat_for_nested, "int a[2][2] = { 1, 2, 3, 4 };");
syntax!(init_struct, "struct S { int a; char b; } s = { 1, 'c' };");
syntax!(
    init_struct_nested,
    "struct I { int x; }; struct O { struct I i; int y; } o = { { 1 }, 2 };"
);
syntax!(init_char_array_from_string, "char s[6] = \"hello\";");
syntax!(init_char_array_inferred, "char s[] = \"hello\";");
syntax!(init_pointer_from_string, "char *s = \"hello\";");
syntax!(init_static_local, "int f(void) { static int a = 1; return a; }");

// ---- 6.5.5 type names ----------------------------------------------------

syntax!(abstract_plain, "int f(void) { return sizeof(int); }");
syntax!(abstract_pointer, "int f(void) { return sizeof(int *); }");
syntax!(abstract_pointer_multi, "int f(void) { return sizeof(char **); }");
syntax!(abstract_array, "int f(void) { return sizeof(int[3]); }");
syntax!(abstract_array_unsized, "void g(int []);");
syntax!(
    abstract_function_pointer,
    "int f(void) { return sizeof(int (*)(void)); }"
);
syntax!(abstract_pointer_to_array, "int f(void) { return sizeof(int (*)[4]); }");
syntax!(abstract_qualified, "int f(void) { return sizeof(const int *); }");
syntax!(
    abstract_struct,
    "struct S { int a; }; int f(void) { return sizeof(struct S); }"
);

// ---- 6.6.1 labeled statements --------------------------------------------

syntax!(label_and_goto, "void f(void) { goto end; end: ; }");
syntax!(
    label_before_declaration_block,
    "void f(void) { a: { int x; x = 0; } goto a; }"
);
syntax!(label_multiple, "void f(void) { a: b: c: ; goto b; }");
syntax!(
    switch_case_default,
    "void f(int x) { switch (x) { case 1: break; case 2: break; default: break; } }"
);
syntax!(
    switch_case_const_expr,
    "void f(int x) { switch (x) { case 1 + 2: break; } }"
);
syntax!(
    switch_fallthrough_and_empty,
    "void f(int x) { switch (x) { case 1: case 2: ; } }"
);
syntax!(
    switch_enum_case,
    "enum E { A, B }; void f(enum E e) { switch (e) { case A: break; default: break; } }"
);

// ---- 6.6.2 compound statements -------------------------------------------

syntax!(compound_empty, "void f(void) { }");
syntax!(compound_declarations_only, "void f(void) { int a; char b; }");
syntax!(compound_statements_only, "void f(void) { ; ; }");
syntax!(compound_declarations_then_statements, "void f(void) { int a; a = 1; }");
syntax!(
    compound_nested_blocks,
    "void f(void) { { int a; { int b; b = 0; a = b; } } }"
);
syntax!(compound_shadowing, "void f(void) { int a; { int a; a = 1; } a = 2; }");

// ---- 6.6.3 expression and null statements --------------------------------

syntax!(statement_null, "void f(void) { ; }");
syntax!(statement_expression, "void f(void) { int a; a = 1; a + 1; }");

// ---- 6.6.4 selection statements ------------------------------------------

syntax!(if_without_else, "void f(int x) { if (x) ; }");
syntax!(if_with_else, "void f(int x) { if (x) ; else ; }");
syntax!(if_else_if_chain, "void f(int x) { if (x) ; else if (x) ; else ; }");
syntax!(if_dangling_else, "void f(int x) { if (x) if (x) ; else ; }");
syntax!(if_with_blocks, "void f(int x) { if (x) { ; } else { ; } }");
syntax!(
    if_declaration_in_branch_block,
    "void f(int x) { if (x) { int y; y = 0; } }"
);

// ---- 6.6.5 iteration statements ------------------------------------------

syntax!(while_loop, "void f(int x) { while (x) ; }");
syntax!(while_with_block, "void f(int x) { while (x) { x = x - 1; } }");
syntax!(do_while_loop, "void f(int x) { do ; while (x); }");
syntax!(do_while_with_block, "void f(int x) { do { x = x - 1; } while (x); }");
syntax!(for_full, "void f(void) { int i; for (i = 0; i < 10; i++) ; }");
syntax!(for_no_init, "void f(void) { int i; i = 0; for (; i < 10; i++) ; }");
syntax!(for_no_condition, "void f(void) { int i; for (i = 0; ; i++) break; }");
syntax!(for_no_increment, "void f(void) { int i; for (i = 0; i < 10;) i++; }");
syntax!(for_empty_all, "void f(void) { for (;;) break; }");
syntax!(
    for_comma_clauses,
    "void f(void) { int i, j; for (i = 0, j = 9; i < j; i++, j--) ; }"
);
syntax!(
    for_nested,
    "void f(void) { int i, j; for (i = 0; i < 2; i++) for (j = 0; j < 2; j++) ; }"
);

// ---- 6.6.6 jump statements -----------------------------------------------

syntax!(
    jump_goto_forward_and_back,
    "void f(void) { int i; i = 0; back: i++; if (i < 2) goto back; goto fwd; fwd: ; }"
);
syntax!(
    jump_continue_break,
    "void f(void) { int i; for (i = 0; i < 10; i++) { if (i) continue; else break; } }"
);
syntax!(jump_break_in_switch, "void f(int x) { switch (x) { default: break; } }");
syntax!(jump_return_void, "void f(void) { return; }");
syntax!(jump_return_value, "int f(void) { return 1; }");
syntax!(jump_return_expression, "int f(int a) { return a ? 1 : 2; }");

// ---- 6.7 external definitions --------------------------------------------

syntax!(external_function_definition, "int f(int a) { return a; }");
syntax!(external_function_no_params, "int f(void) { return 0; }");
syntax!(
    external_multiple_definitions,
    "int f(void) { return 0; } int g(void) { return f(); }"
);
syntax!(
    external_prototype_then_definition,
    "int f(int); int f(int a) { return a; }"
);
syntax!(external_static_definition, "static int f(void) { return 0; }");
syntax!(external_pointer_returning_definition, "int *f(int *p) { return p; }");
syntax!(
    external_struct_param_definition,
    "struct S { int a; }; int f(struct S s) { return s.a; }"
);
syntax!(knr_definition_no_params, "int f() { return 0; }");
syntax!(
    knr_definition_with_params,
    "int f(a, b) int a; int b; { return a + b; }"
);
syntax!(knr_definition_implicit_int, "f(a) int a; { return a; }");
syntax!(knr_definition_pointer_param, "int f(p) char *p; { return *p; }");
syntax!(knr_definition_array_param, "int f(a) int a[]; { return a[0]; }");
syntax!(
    knr_definition_multiple_in_one_decl,
    "int f(a, b) int a, b; { return a + b; }"
);

// ---- typedef interactions across the grammar -----------------------------

syntax!(typedef_pointer, "typedef int *IP; IP p;");
syntax!(typedef_array, "typedef int Arr[10]; Arr a;");
syntax!(typedef_function, "typedef int Fn(int); Fn *fp;");
syntax!(typedef_function_pointer, "typedef int (*Fp)(int); Fp fp;");
syntax!(typedef_struct_anonymous, "typedef struct { int a; } S; S s;");
syntax!(typedef_union, "typedef union { int i; float f; } U; U u;");
syntax!(typedef_enum, "typedef enum { A, B } E; E e;");
syntax!(typedef_qualified, "typedef const int CI; CI a = 1;");
syntax!(typedef_multiple_in_one, "typedef int A, *B, C[4]; A a; B b; C c;");
syntax!(typedef_as_param_type, "typedef int T; int f(T a) { return a; }");
syntax!(typedef_as_return_type, "typedef int T; T f(void) { return 0; }");
syntax!(
    typedef_in_cast_and_sizeof,
    "typedef int T; int f(void) { return (T)1 + sizeof(T); }"
);
syntax!(
    typedef_in_struct_member,
    "typedef int T; struct S { T a; T *b; T c[2]; };"
);
syntax!(
    typedef_abstract_declarator,
    "typedef int T; int f(void) { return sizeof(T *) + sizeof(T (*)(void)); }"
);

// ---- 6.1.4 adjacent string literal concatenation (phase 6) ---------------

literal!(literal_single, "char *s = \"x\";", "x");
literal!(literal_concat_two, "char *s = \"x\" \"y\";", "xy");
literal!(literal_concat_three, "char *s = \"x\" \"y\" \"z\";", "xyz");
literal!(literal_concat_across_comment, "char *s = \"x\" /* c */ \"y\";", "xy");
literal!(literal_concat_across_newline, "char *s = \"x\"\n\"y\"\n\"z\";", "xyz");
literal!(literal_concat_empty_pieces, "char *s = \"\" \"x\" \"\";", "x");
literal!(literal_concat_all_empty, "char *s = \"\" \"\";", "");
literal!(literal_wide_prefix, "int f(void) { return L\"x\"[0]; }", "Lx");
literal!(literal_wide_empty, "int f(void) { return L\"\"[0]; }", "L");

syntax!(concat_in_char_array_init, "char s[] = \"ab\" \"cd\";");
syntax!(concat_in_sized_array_init, "char s[5] = \"ab\" \"cd\";");
syntax!(concat_in_pointer_init, "char *s = \"ab\" \"cd\";");
syntax!(concat_in_array_of_pointers, "char *s[] = { \"a\" \"b\", \"c\" };");
syntax!(
    concat_as_call_argument,
    "int g(char *); int f(void) { return g(\"a\" \"b\"); }"
);
syntax!(concat_in_sizeof, "int f(void) { return sizeof(\"a\" \"b\"); }");
syntax!(concat_in_return, "char *f(void) { return \"a\" \"b\"; }");
syntax!(concat_subscripted, "int f(void) { return (\"ab\" \"cd\")[1]; }");
syntax!(
    concat_in_conditional,
    "char *f(int c) { return c ? \"a\" \"b\" : \"c\" \"d\"; }"
);
syntax!(
    concat_in_struct_init,
    "struct S { char *a; char *b; } s = { \"x\" \"y\", \"z\" };"
);
syntax!(concat_with_escapes, "char *s = \"a\\tb\" \"c\\nd\";");
syntax!(concat_with_escaped_quote, "char *s = \"a\\\"\" \"b\";");
literal!(literal_concat_wide, "int f(void) { return (L\"a\" L\"b\")[0]; }", "Lab");
literal!(
    literal_concat_narrow_then_wide,
    "int f(void) { return (\"a\" L\"b\")[0]; }",
    "Lab"
);
literal!(
    literal_concat_wide_then_narrow,
    "int f(void) { return (L\"a\" \"b\")[0]; }",
    "Lab"
);
syntax!(concat_wide_in_init, "int f(void) { return L\"ab\" \"cd\"[0]; }");

syntax!(splice_in_string, "char *s = \"a\\\nb\";");
syntax!(splice_in_identifier, "int ab; int f(void) { return a\\\nb; }");
syntax!(splice_in_keyword, "in\\\nt x;");
syntax!(
    splice_in_operator,
    "int f(int a) { if (a =\\\n= 1) return 1; return 0; }"
);
syntax!(splice_in_constant, "int f(void) { return 1\\\n2; }");
syntax!(splice_consecutive, "int a\\\n\\\nb;");
syntax!(splice_at_line_start, "int x;\n\\\nint y;");
syntax!(splice_in_declarator, "int *\\\np;");
syntax!(splice_escaped_backslash_before, "char *s = \"a\\\\\\\nb\";");

literal!(literal_splice_in_string, "char *s = \"a\\\nb\";", "ab");
literal!(literal_splice_concat_pieces, "char *s = \"a\" \\\n\"b\";", "ab");

syntax!(knr_implicit_int_no_params, "f() { return 0; }");
syntax!(decl_storage_only, "extern x;");
syntax!(decl_qualifier_then_storage, "const static x;");
syntax!(ptr_two_qualifiers, "int *const volatile p;");
syntax!(sqlist_qualifier_only, "struct S { const a; };");
syntax!(sqlist_two_type_specs, "struct S { unsigned int a; };");
syntax!(sqlist_type_then_qualifier, "struct S { int const a; };");
syntax!(abstract_ptr_then_array, "int f(void) { return sizeof(int *[4]); }");
syntax!(abstract_array_unsized_suffix, "void g(int (*)[]);");
syntax!(abstract_empty_parens, "void g(int ());");
syntax!(abstract_paren_param_list, "void g(int (int));");
