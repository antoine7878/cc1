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

case!(tag_reference_in_inner_scope, "struct S { int a; }; void f(void) { struct S s; }");

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

case!(typedef_of_tag, "struct S { int a; }; typedef struct S S; S v; struct S w;");

// ---- 6.5.3 qualifiers are part of the type -------------------------------

case!(qualifier_conflicting_redeclaration, "const int x; int x;");

case!(qualifier_matching_redeclaration, "const int x; const int x;");

case!(qualifier_volatile_conflicting_redeclaration, "volatile int x; int x;");

case!(qualifier_duplicate_const, "const const int x;");
