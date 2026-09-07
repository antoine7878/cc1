use cc1::semantic::Diagnosis;

// ---- 6.5.2.3 tags name one type across all their mentions ----------------

accept!(tag_reference_after_definition, "struct S { int a; }; struct S x;");

accept!(tag_reference_before_definition, "struct S *p; struct S { int a; };");

accept!(tag_incomplete_pointer, "struct T; struct T *p;");

accept!(tag_self_reference, "struct N { int v; struct N *next; };");

accept!(
    tag_mutually_recursive,
    "struct A { struct B *b; }; struct B { struct A *a; };"
);

reject!(tag_redefinition, "struct S { int a; }; struct S { int b; };");

reject!(tag_kind_mismatch, "struct S { int a; }; union S { int c; };");

accept!(
    tag_reference_in_inner_scope,
    "struct S { int a; }; void f(void) { struct S s; }"
);

accept!(
    tag_shadowed_in_inner_scope,
    "struct S { int a; }; void f(void) { struct S { char b; } s; s.b = 0; }"
);

reject!(tag_duplicate_member, "struct S { int a; int a; };");

accept!(bit_field_width_below_the_type_width, "struct S { int a : 3; };");
accept!(bit_field_width_at_the_type_width, "struct S { int a : 32; };");
accept!(
    unsigned_bit_field_width_at_the_type_width,
    "struct S { unsigned a : 32; };"
);
accept!(
    bit_field_width_from_a_constant_expression,
    "struct S { int a : 2 + 1; };"
);
accept!(anonymous_bit_field_of_zero_width, "struct S { int a : 1; int : 0; };");

reject!(bit_field_width_exceeding_the_type_width, "struct S { int a : 33; };");
reject!(
    unsigned_bit_field_width_exceeding_the_type_width,
    "struct S { unsigned a : 33; };"
);
reject!(
    anonymous_bit_field_width_exceeding_the_type_width,
    "struct S { int a; int : 33; };"
);
reject!(negative_bit_field_width, "struct S { int a : -1; };");
reject!(negative_anonymous_bit_field_width, "struct S { int a; int : -1; };");
reject!(zero_width_named_bit_field, "struct S { int a : 0; };");
reject!(
    bit_field_width_in_a_union_exceeding_the_type_width,
    "union U { int a : 33; };"
);

// ---- 6.5.6 a typedef name is a synonym, not a type of its own -------------

accept!(typedef_repeated_use, "typedef int T; T u; T v;");

accept!(typedef_redeclaration_compatible, "typedef int T; T w; T w;");

accept!(typedef_and_spelled_out_type_agree, "typedef int T; T w; int w;");

// `const P` qualifies the pointer, not the pointee (6.5.3), so `p` is
// `int *const` — the same type as the second declaration.
accept!(
    typedef_qualifier_applies_to_outer_level,
    "typedef int *P; const P p; int *const p;"
);

reject!(typedef_duplicate_qualifier, "typedef const int CI; const CI q;");

accept!(
    typedef_of_tag,
    "struct S { int a; }; typedef struct S S; S v; struct S w;"
);

// ---- 6.5.3 qualifiers are part of the type -------------------------------

reject!(qualifier_conflicting_redeclaration, "const int x; int x;");

accept!(qualifier_matching_redeclaration, "const int x; const int x;");

reject!(qualifier_volatile_conflicting_redeclaration, "volatile int x; int x;");

reject!(qualifier_duplicate_const, "const const int x;");

accept!(
    enum_variant_implicit_increment,
    "enum e { A, B, C }; int f(void) { return C; }"
);

accept!(
    enum_variant_references_previous,
    "enum e { A = 1, B = A + 1, C = B * 2 + A };"
);

accept!(enum_variant_negative_start, "enum e { A = -1, B, C };");

accept!(enum_variant_char_constant, "enum e { A = 'a', B };");

accept!(enum_variant_parenthesized, "enum e { A = (1 + 2) * 3 };");

accept!(enum_variant_int_max, "enum e { M = 2147483647 };");

accept!(enum_variant_sizeof_type, "enum e { A = sizeof(int), B };");

accept!(enum_tag_complete_after_definition, "enum e { A = 1 }; enum e v;");

accept!(
    enum_variant_used_in_function,
    "enum e { A = 1 }; int f(void) { int x; x = A; return x; }"
);

accept!(
    enum_variant_shadowed_in_inner_scope,
    "enum e { A = 1 }; int f(void) { int A; A = 2; return A; }"
);

reject!(enum_variant_undeclared_reference, "enum e { A = Z };");

reject!(enum_variant_references_object, "int x; enum e { A = x };");

reject!(enum_variant_non_integer_constant, "enum e { A = 1.5 };");

reject!(enum_variant_exceeds_int_range, "enum e { M = 2147483647, N };");

reject!(
    identifier_undeclared_in_expression,
    "int f(void) { return undeclared_thing; }"
);

reject!(identifier_used_before_declaration, "int f(void) { return v; }");

recover!(
    enum_recovers_after_undeclared_variant,
    "enum e { A = Z, B = 2, C = B + 1 }; enum e v; int f(void) { return B + C; }",
    [Diagnosis::UndeclaredIdentifier(_), Diagnosis::NonConstantExpression],
    &["B", "C"]
);

recover!(
    enum_recovers_after_non_integer_variant,
    "enum g { P = 1.5, Q = 2, R = 3 }; int f(void) { return Q + R; }",
    [Diagnosis::NonIntegerConstantExpression],
    &["Q", "R"]
);

recover!(
    enum_recovers_after_out_of_range_variant,
    "enum h { M = 2147483647, N, O = 5 }; int g(void) { return O; }",
    [Diagnosis::VariantBadValue],
    &["O"]
);

accept!(identifier_implicit_function_declaration, "int f(void) { return g(); }");

accept!(cast_float_constant_to_int, "enum e { A = (int)1.5 };");

accept!(cast_parenthesized_float_constant, "enum e { A = (int)(1.5) };");

accept!(cast_integral_expression, "enum e { A = (int)(1 + 2) };");

accept!(cast_narrowing_to_char, "enum e { A = (char)300 };");

accept!(cast_narrowing_to_unsigned_char, "enum e { A = (unsigned char)-1 };");

accept!(cast_narrowing_to_short, "enum e { A = (short)70000 };");

accept!(cast_nested_integral, "enum e { A = (int)(char)300 };");

accept!(cast_mixed_integer_ranks, "enum e { A = (long)1 + (short)2 };");

accept!(cast_to_enum_tag, "enum f { X = 1 }; enum e { A = (enum f)2 };");

reject!(cast_to_floating_rejected, "enum e { A = (double)1 };");

reject!(cast_through_floating_rejected, "enum e { A = (int)(double)1 };");

reject!(
    cast_non_immediate_float_operand_rejected,
    "enum e { A = (int)(1.5 + 1) };"
);

reject!(cast_to_pointer_rejected, "enum e { A = (int *)0 };");

reject!(cast_to_void_rejected, "enum e { A = (void)0 };");

reject!(
    cast_to_struct_rejected,
    "struct s { int a; }; enum e { A = (int)(struct s)1 };"
);

reject!(cast_of_object_rejected, "int x; enum e { A = (int)x };");

reject!(
    cast_unsigned_wraparound_exceeds_int_range,
    "enum e { A = (unsigned int)-1 };"
);

value!(
    cast_value_float_truncates_toward_zero,
    "enum e { A = (int)1.5 };",
    &[("A", "1")]
);

value!(cast_value_char_wraps, "enum e { A = (char)300 };", &[("A", "44")]);

value!(cast_value_char_is_signed, "enum e { A = (char)-1 };", &[("A", "-1")]);

value!(
    cast_value_unsigned_char_wraps,
    "enum e { A = (unsigned char)-1 };",
    &[("A", "255")]
);

value!(cast_value_short_wraps, "enum e { A = (short)70000 };", &[("A", "4464")]);

value!(
    cast_value_unsigned_int_wraps,
    "enum e { A = (unsigned int)-1 % 1000 };",
    &[("A", "295")]
);

value!(
    cast_value_long_is_32_bits_on_i386,
    "enum e { A = (unsigned long)-1 % 1000 };",
    &[("A", "295")]
);

value!(
    cast_value_enum_sequence,
    "enum e { A = (int)(1 + 2), B, C = (char)-1, D };",
    &[("A", "3"), ("B", "4"), ("C", "-1"), ("D", "0")]
);

// ---- 6.3.5 the second operand of / and % shall not be zero ---------------

reject!(constant_division_by_zero, "enum E { A = 1 / 0 };");

reject!(constant_remainder_by_zero, "enum E { A = 1 % 0 };");

reject!(constant_division_overflows, "enum E { A = (-2147483647 - 1) / -1 };");

reject!(constant_remainder_overflows, "enum E { A = (-2147483647 - 1) % -1 };");

// ---- 6.1.3.2 the type of an integer constant follows the target ----------

accept!(constant_reaching_unsigned_int, "enum e { A = (0x80000000 % 1000) };");

accept!(constant_sizeof_is_size_t, "enum e { A = sizeof(int) };");

reject!(empty_declaration_of_a_basic_type, "int ;");

reject!(empty_declaration_of_a_qualified_type, "const int;");

reject!(empty_declaration_of_a_typedef_name, "typedef int T; T;");

reject!(empty_declaration_of_an_unnamed_struct, "struct { int a; };");

reject!(empty_declaration_of_an_unnamed_union, "union { int a; };");

accept!(declaration_of_a_struct_tag, "struct S;");

accept!(definition_of_a_struct_tag, "struct S { int a; };");

accept!(definition_of_a_union_tag, "union U { int a; };");

accept!(definition_of_an_enumeration, "enum E { A };");

accept!(definition_of_an_unnamed_enumeration, "enum { A };");

reject!(external_declaration_with_register, "register int x;");

reject!(external_declaration_with_auto, "auto int x;");

reject!(external_definition_with_register, "register int f(void) { return 0; }");

accept!(external_declaration_with_static, "static int x;");

accept!(external_declaration_with_extern, "extern int x;");

accept!(external_declaration_with_typedef, "typedef int T;");

accept!(block_declaration_with_register, "void f(void) { register int x; }");

accept!(block_declaration_with_auto, "void f(void) { auto int x; }");

reject!(
    block_function_declaration_with_auto,
    "void f(void) { auto int g(void); }"
);

reject!(
    block_function_declaration_with_static,
    "void f(void) { static int g(void); }"
);

reject!(
    block_function_declaration_with_register,
    "void f(void) { register int g(); }"
);

reject!(
    nested_block_function_declaration_with_static,
    "void f(void) { { static int g(void); } }"
);

accept!(
    block_function_declaration_with_extern,
    "void f(void) { extern int g(void); }"
);

accept!(
    block_function_declaration_without_storage,
    "void f(void) { int g(void); }"
);

accept!(
    block_declaration_of_a_pointer_to_function,
    "void f(void) { auto int (*fp)(void); }"
);

accept!(
    block_typedef_of_a_function_type,
    "void f(void) { typedef int T(void); }"
);

accept!(external_function_declaration_with_static, "static int g(void);");

accept!(external_function_declaration_returning_pointer, "static int *g(void);");

// ---- 6.5.4.3 function declarators ----------------------------------------

// A function declarator shall not specify a return type that is a function type or an array type.
reject!(function_returning_an_array, "int f(void)[3];");

reject!(
    function_returning_an_array_through_a_typedef,
    "typedef int A[3]; A f(void);"
);

reject!(function_returning_a_function, "int f(void)(void);");

accept!(function_returning_a_pointer_to_function, "int (*f(void))(void);");

accept!(function_returning_a_pointer_to_array, "int (*f(void))[3];");

accept!(
    function_declared_with_an_incomplete_return_type,
    "struct S; struct S f(void);"
);

// ---- 6.5.4.3 compatibility of function types -----------------------------

accept!(function_both_parameter_lists_absent, "int f(); int f();");

accept!(function_prototype_then_unspecified, "int f(int); int f();");

accept!(function_unspecified_then_prototype, "int f(); int f(int);");

accept!(function_void_prototype_then_unspecified, "int f(void); int f();");

accept!(function_pointer_parameter_is_not_promoted, "int f(int *); int f();");

accept!(function_parameter_qualifier_is_dropped, "int f(const int); int f();");

reject!(function_char_parameter_promotes_to_int, "int f(char); int f();");

reject!(function_float_parameter_promotes_to_double, "int f(float); int f();");

reject!(function_variadic_prototype_then_unspecified, "int f(int,...); int f();");

accept!(
    function_prototype_then_identifier_list_definition,
    "int f(int); int f(a) int a; { }"
);

accept!(
    function_unspecified_then_identifier_list_definition,
    "int f(); int f(a) int a; { }"
);

accept!(
    function_identifier_without_declaration_is_int,
    "int f(int); int f(a) { }"
);

accept!(
    function_identifier_short_promotes_to_int,
    "int f(int); int f(a) short a; { }"
);

accept!(
    function_identifier_float_promotes_to_double,
    "int f(double); int f(a) float a; { }"
);

accept!(
    function_variadic_prototype_then_identifier_list_definition,
    "int f(int,...); int f(a) int a; { }"
);

accept!(
    function_identifier_list_definition_then_prototype,
    "int f(a) int a; { } int f(int);"
);

reject!(
    function_prototype_then_empty_identifier_list_definition,
    "int f(int); int f() { }"
);

reject!(
    function_identifier_list_definition_with_fewer_parameters,
    "int f(int,int); int f(a) int a; { }"
);

reject!(
    function_identifier_char_promotes_to_int,
    "int f(char); int f(a) char a; { }"
);

// ---- 6.5.2.2 an enumerated type is compatible with an integer type --------

accept!(
    enum_parameter_prototype_then_unspecified,
    "enum e { A }; int f(enum e); int f();"
);

accept!(
    enum_parameter_prototype_then_identifier_list_definition,
    "enum e { A }; int f(enum e); int f(a) enum e a; { }"
);

reject!(
    function_identifier_list_definition_with_other_return_type,
    "int f(int); void f(a) int a; { }"
);

// ---- 6.5.3 a qualifier is part of the type on every level ----------------

reject!(compatible_const_against_plain, "extern const int x; extern int x;");

reject!(
    compatible_volatile_against_plain,
    "extern volatile int x; extern int x;"
);

reject!(
    compatible_const_pointer_against_plain,
    "extern int *const p; extern int *p;"
);

reject!(
    compatible_pointer_to_const_against_pointer,
    "extern const int *p; extern int *p;"
);

reject!(
    compatible_const_tag_against_plain,
    "struct S { int a; }; extern const struct S s; extern struct S s;"
);

reject!(
    compatible_const_element_against_plain,
    "extern const int a[10]; extern int a[10];"
);

// ---- 6.5.4.1 pointers to compatible types --------------------------------

accept!(
    compatible_pointer_to_function,
    "extern int (*p)(int); extern int (*p)(int);"
);

reject!(
    compatible_pointer_to_function_other_parameter,
    "extern int (*p)(int); extern int (*p)(char);"
);

reject!(
    compatible_pointer_to_function_other_return,
    "extern int (*p)(int); extern char (*p)(int);"
);

accept!(
    compatible_pointer_to_completed_tag,
    "struct S; extern struct S *p; struct S { int a; }; extern struct S *p;"
);

// ---- 6.5.4.2 array sizes agree only when both are present ----------------

accept!(compatible_array_unsized_then_sized, "extern int a[]; extern int a[10];");

accept!(compatible_array_sized_then_unsized, "extern int a[10]; extern int a[];");

accept!(
    compatible_array_both_unsized,
    "extern const int a[]; extern const int a[10];"
);

reject!(compatible_array_other_size, "extern int a[10]; extern int a[11];");

accept!(
    compatible_array_of_array_outer_unsized,
    "extern int a[][4]; extern int a[3][4];"
);

accept!(
    compatible_array_of_array_outer_sized_first,
    "extern int a[10][4]; extern int a[][4];"
);

reject!(
    compatible_array_of_array_other_inner_size,
    "extern int a[3][4]; extern int a[3][5];"
);

accept!(
    compatible_pointer_to_unsized_array,
    "extern int (*p)[10]; extern int (*p)[];"
);

// ---- 6.5.2.2 an enumerated type is compatible with one integer type ------

accept!(
    compatible_enum_declared_twice,
    "enum a { X }; extern enum a v; extern enum a v;"
);

reject!(
    compatible_distinct_enums,
    "enum a { X }; enum b { Y }; extern enum a v; extern enum b v;"
);

// ---- 6.1.2.5 the character and integer types are distinct types ----------

reject!(
    compatible_char_against_signed_char,
    "extern char c; extern signed char c;"
);

reject!(
    compatible_int_against_unsigned_int,
    "extern int x; extern unsigned int x;"
);

accept!(compatible_int_against_signed_int, "extern int x; extern signed int x;");

// ---- 6.5 an object with no linkage must be complete by end of declarator -

reject!(incomplete_void_variable, "void v;");

reject!(incomplete_initialized_variable, "struct S; struct S x = { 1 };");

reject!(incomplete_block_variable, "struct S; void f(void) { struct S x; }");

reject!(
    incomplete_static_block_variable,
    "struct S; void f(void) { static struct S x; }"
);

reject!(incomplete_array_element_sized, "struct S; struct S a[3];");

reject!(incomplete_array_element_unsized, "struct S; extern struct S a[];");

reject!(void_array_element, "void a[3];");

// ---- 6.5.2.1 a structure or union shall not contain a member with incomplete or function type ----

reject!(incomplete_struct_member, "struct S; struct T { struct S s; };");

reject!(void_struct_member, "struct U { void v; };");

reject!(self_referential_struct_member, "struct W { struct W w; };");

reject!(
    self_referential_struct_member_no_crash,
    "struct W { struct W w; }; enum e { P = sizeof(struct W) };"
);

reject!(
    self_referential_struct_array_member_zero_size_no_crash,
    "struct W { struct W w[0]; }; enum e { P = sizeof(struct W) };"
);

reject!(
    self_referential_struct_array_member_no_crash,
    "struct W { struct W w[3]; }; enum e { P = sizeof(struct W) };"
);

// ---- 6.7.1 the resulting parameter type shall be an object type ----------

reject!(
    incomplete_parameter_prototype,
    "struct S; void h(struct S p) { (void)0; }"
);

reject!(
    incomplete_parameter_old_style,
    "struct S; void h(p) struct S p; { (void)0; }"
);

// ---- 6.7.1 a function definition's return type shall be void or complete -

reject!(
    incomplete_return_type,
    "struct S; struct S r(void) { struct S v; return v; }"
);

reject!(incomplete_definition_parameter_array, "struct S; void f(struct S a[]);");

accept!(extern_incomplete_variable, "struct S; extern struct S b;");

accept!(
    extern_incomplete_block_variable,
    "struct S; void f(void) { extern struct S y; }"
);

accept!(pointer_to_incomplete_variable, "struct S; struct S *p;");

accept!(typedef_of_incomplete_type, "struct S; typedef struct S TS;");

accept!(incomplete_parameter_in_prototype_only, "struct S; void g(struct S p);");

accept!(incomplete_return_in_prototype_only, "struct S; struct S ret(void);");

accept!(self_referential_struct_pointer_member, "struct X { struct X *p; };");

accept!(
    tentative_definition_completed_later,
    "struct S; struct S late; struct S { int a; };"
);

accept!(unsized_array_completed_by_initializer, "int a[] = { 1, 2 };");

recover!(
    two_dimensional_array_reports_element_incompleteness_once,
    "struct S; struct S a[2][3];",
    [Diagnosis::InvalidElementType(_)],
    &[]
);

recover!(
    incomplete_member_reports_once_without_tag_without_member,
    "struct S; struct T { struct S s; };",
    [Diagnosis::InvalidMemberType(_)],
    &[]
);
