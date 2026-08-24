mod common;

// ---- 6.5.2.3 tags name one type across all their mentions ----------------

case!(tag_reference_after_definition, "struct S { int a; }; struct S x;");

case!(tag_reference_before_definition, "struct S *p; struct S { int a; };");

case!(tag_incomplete_pointer, "struct T; struct T *p;");

case!(tag_self_reference, "struct N { int v; struct N *next; };");

case!(
    tag_mutually_recursive,
    "struct A { struct B *b; }; struct B { struct A *a; };"
);

case!(tag_redefinition, "struct S { int a; }; struct S { int b; };");

case!(tag_kind_mismatch, "struct S { int a; }; union S { int c; };");

case!(
    tag_reference_in_inner_scope,
    "struct S { int a; }; void f(void) { struct S s; }"
);

case!(
    tag_shadowed_in_inner_scope,
    "struct S { int a; }; void f(void) { struct S { char b; } s; s.b = 0; }"
);

case!(tag_duplicate_member, "struct S { int a; int a; };");

// ---- 6.5.6 a typedef name is a synonym, not a type of its own -------------

case!(typedef_repeated_use, "typedef int T; T u; T v;");

case!(typedef_redeclaration_compatible, "typedef int T; T w; T w;");

case!(typedef_and_spelled_out_type_agree, "typedef int T; T w; int w;");

// `const P` qualifies the pointer, not the pointee (6.5.3), so `p` is
// `int *const` — the same type as the second declaration.
case!(
    typedef_qualifier_applies_to_outer_level,
    "typedef int *P; const P p; int *const p;"
);

case!(typedef_duplicate_qualifier, "typedef const int CI; const CI q;");

case!(
    typedef_of_tag,
    "struct S { int a; }; typedef struct S S; S v; struct S w;"
);

// ---- 6.5.3 qualifiers are part of the type -------------------------------

case!(qualifier_conflicting_redeclaration, "const int x; int x;");

case!(qualifier_matching_redeclaration, "const int x; const int x;");

case!(qualifier_volatile_conflicting_redeclaration, "volatile int x; int x;");

case!(qualifier_duplicate_const, "const const int x;");

case!(
    enum_variant_implicit_increment,
    "enum e { A, B, C }; int f(void) { return C; }"
);

case!(
    enum_variant_references_previous,
    "enum e { A = 1, B = A + 1, C = B * 2 + A };"
);

case!(enum_variant_negative_start, "enum e { A = -1, B, C };");

case!(enum_variant_char_constant, "enum e { A = 'a', B };");

case!(enum_variant_parenthesized, "enum e { A = (1 + 2) * 3 };");

case!(enum_variant_int_max, "enum e { M = 2147483647 };");

case!(enum_variant_sizeof_type, "enum e { A = sizeof(int), B };");

case!(enum_tag_complete_after_definition, "enum e { A = 1 }; enum e v;");

case!(
    enum_variant_used_in_function,
    "enum e { A = 1 }; int f(void) { int x; x = A; return x; }"
);

case!(
    enum_variant_shadowed_in_inner_scope,
    "enum e { A = 1 }; int f(void) { int A; A = 2; return A; }"
);

case!(enum_variant_undeclared_reference, "enum e { A = Z };");

case!(enum_variant_references_object, "int x; enum e { A = x };");

case!(enum_variant_non_integer_constant, "enum e { A = 1.5 };");

case!(enum_variant_exceeds_int_range, "enum e { M = 2147483647, N };");

case!(
    identifier_undeclared_in_expression,
    "int f(void) { return undeclared_thing; }"
);

case!(identifier_used_before_declaration, "int f(void) { return v; }");

recover!(
    enum_recovers_after_undeclared_variant,
    "enum e { A = Z, B = 2, C = B + 1 }; enum e v; int f(void) { return B + C; }",
    1,
    &["'B'", "'C'"]
);

recover!(
    enum_recovers_after_non_integer_variant,
    "enum g { P = 1.5, Q = 2, R = 3 }; int f(void) { return Q + R; }",
    1,
    &["'Q'", "'R'"]
);

recover!(
    enum_recovers_after_out_of_range_variant,
    "enum h { M = 2147483647, N, O = 5 }; int g(void) { return O; }",
    1,
    &["'O'"]
);

// case!(identifier_implicit_function_declaration, "int f(void) { return g(); }");

case!(cast_float_constant_to_int, "enum e { A = (int)1.5 };");

case!(cast_parenthesized_float_constant, "enum e { A = (int)(1.5) };");

case!(cast_integral_expression, "enum e { A = (int)(1 + 2) };");

case!(cast_narrowing_to_char, "enum e { A = (char)300 };");

case!(cast_narrowing_to_unsigned_char, "enum e { A = (unsigned char)-1 };");

case!(cast_narrowing_to_short, "enum e { A = (short)70000 };");

case!(cast_nested_integral, "enum e { A = (int)(char)300 };");

case!(cast_mixed_integer_ranks, "enum e { A = (long)1 + (short)2 };");

case!(cast_to_enum_tag, "enum f { X = 1 }; enum e { A = (enum f)2 };");

case!(cast_to_floating_rejected, "enum e { A = (double)1 };");

case!(cast_through_floating_rejected, "enum e { A = (int)(double)1 };");

case!(cast_non_immediate_float_operand_rejected, "enum e { A = (int)(1.5 + 1) };");

case!(cast_to_pointer_rejected, "enum e { A = (int *)0 };");

case!(cast_to_void_rejected, "enum e { A = (void)0 };");

case!(cast_to_struct_rejected, "struct s { int a; }; enum e { A = (int)(struct s)1 };");

case!(cast_of_object_rejected, "int x; enum e { A = (int)x };");

case!(cast_unsigned_wraparound_exceeds_int_range, "enum e { A = (unsigned int)-1 };");

value!(cast_value_float_truncates_toward_zero, "enum e { A = (int)1.5 };", &[("A", "1")]);

value!(cast_value_char_wraps, "enum e { A = (char)300 };", &[("A", "44")]);

value!(cast_value_char_is_signed, "enum e { A = (char)-1 };", &[("A", "-1")]);

value!(cast_value_unsigned_char_wraps, "enum e { A = (unsigned char)-1 };", &[("A", "255")]);

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
