use cc1::semantic::CastKind::*;
use cc1::semantic::Diagnosis;
use crate::common::{Shape, Ty, Unit, ints, lv, none, rv};

macro_rules! shaped {
    ($name:ident, $src:expr, $shapes:expr $(,)?) => {
        #[test]
        fn $name() {
            let unit = Unit::compile($src);
            assert!(unit.parsed(), "cc1 failed to parse:\n{}", $src);
            assert!(
                unit.diagnosis().is_empty(),
                "unexpected diagnosis:\n{}\n{}",
                $src,
                unit.render()
            );
            let expected: Vec<Shape> = $shapes;
            assert_eq!(unit.shapes(), expected, "{}", $src);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr, $shapes:expr $(,)?) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            let unit = Unit::compile($src);
            assert!(unit.parsed(), "cc1 failed to parse:\n{}", $src);
            let expected: Vec<Shape> = $shapes;
            assert_eq!(unit.shapes(), expected, "{}", $src);
        }
    };
}

macro_rules! rejects_shaped {
    (@build $name:ident, $src:expr, $u:ident, $diagnosis:pat, $guard:expr, $shapes:expr) => {
        #[test]
        fn $name() {
            let $u = Unit::compile($src);
            assert!($u.parsed(), "cc1 failed to parse:\n{}", $src);
            let got: Vec<_> = $u.diagnosis().iter().map(|diag| diag.inner.clone()).collect();
            assert!(
                matches!(got.as_slice(), [$diagnosis] if $guard),
                "expected one {}, got {got:?}:\n{}\n{}",
                stringify!($diagnosis),
                $src,
                $u.render()
            );
            let expected: Vec<Shape> = $shapes;
            assert_eq!($u.shapes(), expected, "{}", $src);
        }
    };
    ($name:ident, $src:expr, |$u:ident| $diagnosis:pat if $guard:expr, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, $u, $diagnosis, $guard, $shapes);
    };
    ($name:ident, $src:expr, $diagnosis:pat, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, _u, $diagnosis, true, $shapes);
    };
}

// ---- 6.3.1 primary expressions -------------------------------------------

shaped!(a_constant_is_an_rvalue, "void f(void) { 1; }", vec![rv(Ty::Int)]);

shaped!(
    an_identifier_is_an_lvalue,
    "int i; void f(void) { i; }",
    vec![lv(Ty::Int)]
);

shaped!(
    an_enumeration_constant_is_an_rvalue,
    "enum E { A }; void f(void) { A; }",
    vec![rv(Ty::Int)]
);

shaped!(
    a_string_literal_is_an_array_lvalue,
    "void f(void) { \"ab\"; }",
    vec![lv(Ty::arr(Ty::Char, 3))]
);

// ---- 6.2.2.1 lvalue, array and function conversions ----------------------

shaped!(
    an_operand_is_converted_to_the_value_it_designates,
    "int i; void f(void) { -i; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

rejects_shaped!(
    an_array_operand_becomes_a_pointer_to_its_first_element,
    "char a[10]; void f(void) { -a; }",
    Diagnosis::InvalidOperand,
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Char, 10)).then(ArrayToPointer, Ty::ptr(Ty::Char)),
        none(),
    ]
);

rejects_shaped!(
    a_function_designator_becomes_a_pointer_to_function,
    "int g(); void f(void) { g + 1; }",
    Diagnosis::InvalidOperand,
    vec![
        rv(Ty::noproto(Ty::Int)).then(FunctionToPointer, Ty::ptr(Ty::noproto(Ty::Int))),
        rv(Ty::Int),
        none(),
    ]
);

// ---- 6.2.1.1 integral promotions -----------------------------------------

shaped!(
    a_char_operand_promotes_to_int,
    "char c; void f(void) { -c; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_short_operand_promotes_to_int,
    "short s; void f(void) { -s; }",
    vec![
        lv(Ty::Short)
            .then(LValueToRValue, Ty::Short)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_double_operand_is_left_alone,
    "double d; void f(void) { -d; }",
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), rv(Ty::Double)]
);

shaped!(
    both_operands_of_an_addition_promote,
    "char c; void f(void) { c + c; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

// ---- 6.2.1.5 usual arithmetic conversions --------------------------------

shaped!(two_constants_stay_int, "void f(void) { 1 + 2; }", ints(3));

shaped!(
    an_integer_and_a_double_meet_at_double,
    "int i; double d; void f(void) { i + d; }",
    vec![
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToFloating, Ty::Double),
        lv(Ty::Double).then(LValueToRValue, Ty::Double),
        rv(Ty::Double),
    ]
);

shaped!(
    an_integer_and_a_float_meet_at_float,
    "float g; int i; void f(void) { g + i; }",
    vec![
        lv(Ty::Float).then(LValueToRValue, Ty::Float),
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToFloating, Ty::Float),
        rv(Ty::Float),
    ]
);

shaped!(
    an_int_converts_to_the_unsigned_int_it_meets,
    "unsigned u; int i; void f(void) { u + i; }",
    vec![
        lv(Ty::UInt).then(LValueToRValue, Ty::UInt),
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerConversion, Ty::UInt),
        rv(Ty::UInt),
    ]
);

// ---- 6.3.3.3 unary arithmetic operators ----------------------------------

rejects_shaped!(
    minus_rejects_a_pointer,
    "int *p; void f(void) { -p; }",
    Diagnosis::InvalidOperand,
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), none()]
);

rejects_shaped!(
    minus_rejects_a_structure,
    "struct S { int x; } s; void f(void) { -s; }",
    Diagnosis::InvalidOperand,
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

// ---- 6.3.6 additive operators --------------------------------------------

rejects_shaped!(
    add_rejects_two_structures,
    "struct S { int x; } s; void f(void) { s + s; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        none(),
    ]
);

rejects_shaped!(
    add_rejects_a_floating_index,
    "int *p; void f(void) { p + 1.5; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Double),
        none(),
    ]
);

rejects_shaped!(
    add_rejects_a_pointer_to_void,
    "void *p; void f(void) { p + 1; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    add_rejects_a_pointer_to_an_incomplete_type,
    "struct S *p; void f(void) { p + 1; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        rv(Ty::Int),
        none(),
    ]
);

shaped!(
    a_pointer_plus_an_integer_is_a_pointer,
    "int *p; void f(void) { p + 1; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    an_integer_plus_a_pointer_is_a_pointer,
    "int *p; void f(void) { 1 + p; }",
    vec![
        rv(Ty::Int),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    an_array_plus_an_integer_is_a_pointer_to_the_element,
    "char a[10]; void f(void) { a + 1; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Char, 10)).then(ArrayToPointer, Ty::ptr(Ty::Char)),
        rv(Ty::Int),
        rv(Ty::ptr(Ty::Char)),
    ]
);

// ---- 6.3.4 cast operators --------------------------------------------------

shaped!(
    a_cast_result_is_always_an_rvalue,
    "int i; void f(void) { (double)i; }",
    vec![
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToFloating, Ty::Double),
        rv(Ty::Double),
    ]
);

shaped!(
    a_pointer_may_be_cast_to_an_integer,
    "int *p; void f(void) { (int)p; }",
    vec![
        lv(Ty::ptr(Ty::Int))
            .then(LValueToRValue, Ty::ptr(Ty::Int))
            .then(PointerToInteger, Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    an_integer_may_be_cast_to_a_pointer,
    "int i; void f(void) { (int *)i; }",
    vec![
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToPointer, Ty::ptr(Ty::Int)),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    a_cast_to_void_discards_the_value,
    "void f(void) { (void)1; }",
    vec![rv(Ty::Int), rv(Ty::Void)]
);

rejects_shaped!(
    cast_of_a_non_scalar_operand_is_rejected,
    "struct S { int a; } s; void f(void) { (int)s; }",
    Diagnosis::CastToNonScalar,
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

rejects_shaped!(
    cast_to_a_non_scalar_type_is_rejected,
    "struct S { int a; }; void f(void) { (struct S)1; }",
    Diagnosis::CastToNonScalar,
    vec![rv(Ty::Int), none()]
);

rejects_shaped!(
    cast_from_a_pointer_to_a_floating_type_is_rejected,
    "int *p; void f(void) { (double)p; }",
    Diagnosis::InvalidOperand,
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), none()]
);

rejects_shaped!(
    cast_from_a_floating_type_to_a_pointer_is_rejected,
    "double d; void f(void) { (int *)d; }",
    Diagnosis::InvalidOperand,
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), none()]
);

#[test]
fn cast_to_a_non_scalar_type_is_reported_once() {
    let unit = Unit::compile("struct S { int a; }; enum E { A = (struct S)1 };");
    assert!(unit.parsed(), "cc1 failed to parse:\n{}", unit.render());
    let got: Vec<_> = unit.diagnosis().iter().map(|diag| diag.inner.clone()).collect();
    assert!(
        matches!(got.as_slice(), [Diagnosis::CastToNonScalar]),
        "expected exactly one CastToNonScalar, got {got:?}:\n{}",
        unit.render()
    );
}

// ---- 6.4 constant expressions --------------------------------------------

shaped!(a_constant_expression_mirrors_its_operand, "int a[2 + 3];", ints(4));

// ---- 6.3.6 additive operators: subtraction --------------------------------

shaped!(
    a_pointer_minus_an_integer_is_a_pointer,
    "int *p; void f(void) { p - 1; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    two_compatible_pointers_subtract_to_ptrdiff_t,
    "int *p, *q; void f(void) { p - q; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

shaped!(
    pointer_subtraction_works_for_any_compatible_object_type,
    "char *p, *q; void f(void) { p - q; }",
    vec![
        lv(Ty::ptr(Ty::Char)).then(LValueToRValue, Ty::ptr(Ty::Char)),
        lv(Ty::ptr(Ty::Char)).then(LValueToRValue, Ty::ptr(Ty::Char)),
        rv(Ty::Int),
    ]
);

rejects_shaped!(
    an_integer_minus_a_pointer_is_rejected,
    "int i; int *p; void f(void) { i - p; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        none(),
    ]
);

rejects_shaped!(
    subtracting_pointers_to_an_incomplete_type_is_rejected,
    "struct S *p, *q; void f(void) { p - q; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        none(),
    ]
);

rejects_shaped!(
    subtracting_a_floating_index_is_rejected,
    "int *p; void f(void) { p - 1.5; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Double),
        none(),
    ]
);

// ---- 6.3.16.1 simple assignment --------------------------------------------

shaped!(
    assigning_a_constant_keeps_the_lvalues_type,
    "int x; void f(void) { x = 1; }",
    vec![lv(Ty::Int), rv(Ty::Int), rv(Ty::Int)]
);

shaped!(
    assignment_converts_the_right_operand_to_the_left_operands_type,
    "int x; void f(void) { x = 3.5; }",
    vec![
        lv(Ty::Int),
        rv(Ty::Double).then(FloatingToInteger, Ty::Int),
        rv(Ty::Int)
    ]
);

shaped!(
    zero_is_a_null_pointer_constant,
    "int *p; void f(void) { p = 0; }",
    vec![
        lv(Ty::ptr(Ty::Int)),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int)),
        rv(Ty::ptr(Ty::Int)),
    ]
);

rejects_shaped!(
    assigning_an_incompatible_pointer_is_rejected,
    "char *p; int *q; void f(void) { q = p; }",
    |u| Diagnosis::AssignmentIncompatibleTypes(to, from)
        if *to == u.symbol_ty("q") && *from == u.symbol_ty("p"),
    vec![
        lv(Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::Char)).then(LValueToRValue, Ty::ptr(Ty::Char)),
        none(),
    ]
);

rejects_shaped!(
    assigning_away_const_through_a_pointer_is_rejected,
    "const char *p; char *q; void f(void) { q = p; }",
    Diagnosis::AssignmentDiscardedQualifiers(_),
    vec![
        lv(Ty::ptr(Ty::Char)),
        lv(Ty::ptr(Ty::konst(Ty::Char))).then(LValueToRValue, Ty::ptr(Ty::konst(Ty::Char))),
        none(),
    ]
);

shaped!(
    assigning_a_pointer_that_gains_const_is_accepted,
    "char *p; const char *q; void f(void) { q = p; }",
    vec![
        lv(Ty::ptr(Ty::konst(Ty::Char))),
        lv(Ty::ptr(Ty::Char))
            .then(LValueToRValue, Ty::ptr(Ty::Char))
            .then(PointerConversion, Ty::ptr(Ty::konst(Ty::Char))),
        rv(Ty::ptr(Ty::konst(Ty::Char))),
    ]
);

shaped!(
    a_compatible_structure_may_be_assigned,
    "struct S { int a; } x, y; void f(void) { x = y; }",
    vec![
        lv(Ty::strukt("S")),
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        rv(Ty::strukt("S")),
    ]
);

rejects_shaped!(
    assigning_to_an_rvalue_is_rejected,
    "void f(void) { 1 = 1; }",
    Diagnosis::AssignToRValue,
    vec![rv(Ty::Int), rv(Ty::Int), none()]
);

rejects_shaped!(
    assigning_to_a_const_variable_is_rejected,
    "void f(void) { const int x; x = 1; }",
    Diagnosis::ConstAssignment,
    vec![lv(Ty::konst(Ty::Int)), rv(Ty::Int), none()]
);

// ---- 6.5.7 initialization ---------------------------------------------

shaped!(
    initializing_with_a_constant_keeps_its_type,
    "void f(void) { int x = 1; }",
    vec![rv(Ty::Int)]
);

shaped!(
    initialization_converts_the_initializer_to_the_declared_type,
    "void f(void) { int x = 3.5; }",
    vec![rv(Ty::Double).then(FloatingToInteger, Ty::Int)]
);

shaped!(
    zero_initializes_a_pointer_as_a_null_pointer_constant,
    "void f(void) { int *p = 0; }",
    vec![rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int))]
);

rejects_shaped!(
    initializing_a_pointer_with_a_double_is_rejected,
    "void f(void) { int *p = 3.5; }",
    |u| Diagnosis::InitIncompatibleTypes(to, from)
        if *to == u.symbol_ty("p") && *from == u.prim("double"),
    vec![rv(Ty::Double)]
);

rejects_shaped!(
    initializing_with_an_incompatible_pointer_is_rejected,
    "void f(void) { char *p; int *q = p; }",
    |u| Diagnosis::InitIncompatibleTypes(to, from)
        if *to == u.symbol_ty("q") && *from == u.symbol_ty("p"),
    vec![lv(Ty::ptr(Ty::Char)).then(LValueToRValue, Ty::ptr(Ty::Char))]
);

rejects_shaped!(
    initializing_away_const_through_a_pointer_is_rejected,
    "void f(void) { const char *p; char *q = p; }",
    Diagnosis::InitDiscardedQualifiers(_),
    vec![lv(Ty::ptr(Ty::konst(Ty::Char))).then(LValueToRValue, Ty::ptr(Ty::konst(Ty::Char))),]
);

shaped!(
    initializing_a_pointer_that_gains_const_is_accepted,
    "void f(void) { char *p; const char *q = p; }",
    vec![
        lv(Ty::ptr(Ty::Char))
            .then(LValueToRValue, Ty::ptr(Ty::Char))
            .then(PointerConversion, Ty::ptr(Ty::konst(Ty::Char))),
    ]
);

// ---- 6.5.7 array and scalar initializer lists -----------------------------

shaped!(
    array_initializer_types_each_element,
    "void f(void) { int a[3] = {1,2,3}; }",
    ints(5)
);

shaped!(
    array_initializer_may_have_fewer_elements_than_declared,
    "void f(void) { int a[3] = {1,2}; }",
    ints(4)
);

rejects_shaped!(
    array_initializer_with_too_many_elements_is_rejected,
    "void f(void) { int a[2] = {1,2,3}; }",
    Diagnosis::ArrayInitTooLong,
    ints(4)
);

shaped!(
    incomplete_array_size_is_inferred_from_initializer,
    "void f(void) { int a[] = {1,2,3}; }",
    ints(3)
);

shaped!(
    scalar_initializer_may_be_wrapped_in_braces,
    "void f(void) { int x = {1}; }",
    ints(1)
);

rejects_shaped!(
    scalar_initializer_with_too_many_elements_is_rejected,
    "void f(void) { int x = {1,2}; }",
    Diagnosis::ArrayInitTooLong,
    ints(1)
);

shaped!(
    nested_array_initializer_types_every_element,
    "void f(void) { int a[2][2] = {{1,2},{3,4}}; }",
    ints(8)
);

// Regression test: an excess element in one inner list must not corrupt the
// element type used for a later sibling list.
rejects_shaped!(
    excess_elements_in_a_nested_list_do_not_affect_sibling_lists,
    "void f(void) { int a[2][2] = {{1,2,3},{4,5}}; }",
    Diagnosis::ArrayInitTooLong,
    ints(8)
);

shaped!(
    array_initializer_converts_each_element_to_the_declared_type,
    "void f(void) { int a[2] = {1, 3.5}; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Double).then(FloatingToInteger, Ty::Int),
    ]
);

shaped!(
    array_of_pointers_initializer_accepts_null_pointer_constants,
    "void f(void) { int *a[2] = {0, 0}; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int)),
    ]
);

// ---- resolution failures do not cascade ----------------------------------

rejects_shaped!(
    an_undeclared_identifier_has_no_type,
    "void f(void) { x; }",
    Diagnosis::UndeclaredIdentifier(_),
    vec![none()]
);

rejects_shaped!(
    an_operand_without_a_type_is_reported_once,
    "void f(void) { x + 1; }",
    Diagnosis::UndeclaredIdentifier(_),
    vec![none(), rv(Ty::Int), none()]
);
