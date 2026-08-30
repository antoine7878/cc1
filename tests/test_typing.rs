mod common;

use cc1::semantic::Diagnosis;
use common::Unit;

fn typed(src: &str) -> Vec<String> {
    let unit = Unit::compile(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    unit.typed()
}

/// Every resolved expression of the unit, in source order, rendered as its type as written,
/// its value category, then the conversions it acquires on the way to its parent.
macro_rules! types {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(typed($src), $expected, "{}", $src);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            assert_eq!(typed($src), $expected, "{}", $src);
        }
    };
}

/// A unit that reports exactly one diagnosis. The operands keep the conversions they acquired
/// before the constraint was checked, and the rejected expression itself carries no type.
macro_rules! rejects {
    ($name:ident, $src:expr, $diagnosis:pat, $expected:expr) => {
        #[test]
        fn $name() {
            let unit = Unit::compile($src);
            assert!(unit.parsed(), "cc1 failed to parse:\n{}", $src);
            let got: Vec<_> = unit.diagnosis().iter().map(|diag| diag.inner).collect();
            assert!(
                matches!(got.as_slice(), [$diagnosis]),
                "expected one {}, got {got:?}:\n{}\n{}",
                stringify!($diagnosis),
                $src,
                unit.render()
            );
            assert_eq!(unit.typed(), $expected, "{}", $src);
        }
    };
}

// ---- 6.3.1 primary expressions -------------------------------------------

types!(a_constant_is_an_rvalue, "void f(void) { 1; }", ["int"]);

types!(an_identifier_is_an_lvalue, "int i; void f(void) { i; }", ["int lvalue"]);

types!(
    an_enumeration_constant_is_an_rvalue,
    "enum E { A }; void f(void) { A; }",
    ["int"]
);

types!(
    a_string_literal_is_an_array_lvalue,
    "void f(void) { \"ab\"; }",
    ["char[3] lvalue"]
);

// ---- 6.2.2.1 lvalue, array and function conversions ----------------------

types!(
    an_operand_is_converted_to_the_value_it_designates,
    "int i; void f(void) { -i; }",
    ["int lvalue <LValueToRValue> int", "int"]
);

rejects!(
    an_array_operand_becomes_a_pointer_to_its_first_element,
    "char a[10]; void f(void) { -a; }",
    Diagnosis::InvalidOperand,
    ["int", "int", "char[10] lvalue <ArrayToPointer> *char", "None"]
);

rejects!(
    a_function_designator_becomes_a_pointer_to_function,
    "int g(); void f(void) { g + 1; }",
    Diagnosis::InvalidOperand,
    ["int() <FunctionToPointer> *(int())", "int", "None"]
);

// ---- 6.2.1.1 integral promotions -----------------------------------------

types!(
    a_char_operand_promotes_to_int,
    "char c; void f(void) { -c; }",
    ["char lvalue <LValueToRValue> char <IntegerPromotion> int", "int"]
);

types!(
    a_short_operand_promotes_to_int,
    "short s; void f(void) { -s; }",
    ["short lvalue <LValueToRValue> short <IntegerPromotion> int", "int"]
);

types!(
    a_double_operand_is_left_alone,
    "double d; void f(void) { -d; }",
    ["double lvalue <LValueToRValue> double", "double"]
);

types!(
    both_operands_of_an_addition_promote,
    "char c; void f(void) { c + c; }",
    [
        "char lvalue <LValueToRValue> char <IntegerPromotion> int",
        "char lvalue <LValueToRValue> char <IntegerPromotion> int",
        "int"
    ]
);

// ---- 6.2.1.5 usual arithmetic conversions --------------------------------

types!(two_constants_stay_int, "void f(void) { 1 + 2; }", ["int", "int", "int"]);

types!(
    an_integer_and_a_double_meet_at_double,
    "int i; double d; void f(void) { i + d; }",
    [
        "int lvalue <LValueToRValue> int <IntegerToFloating> double",
        "double lvalue <LValueToRValue> double",
        "double"
    ]
);

types!(
    an_integer_and_a_float_meet_at_float,
    "float g; int i; void f(void) { g + i; }",
    [
        "float lvalue <LValueToRValue> float",
        "int lvalue <LValueToRValue> int <IntegerToFloating> float",
        "float"
    ]
);

types!(
    an_int_converts_to_the_unsigned_int_it_meets,
    "unsigned u; int i; void f(void) { u + i; }",
    [
        "unsigned int lvalue <LValueToRValue> unsigned int",
        "int lvalue <LValueToRValue> int <IntegerConversion> unsigned int",
        "unsigned int"
    ]
);

// ---- 6.3.3.3 unary arithmetic operators ----------------------------------

rejects!(
    minus_rejects_a_pointer,
    "int *p; void f(void) { -p; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "None"]
);

rejects!(
    minus_rejects_a_structure,
    "struct S { int x; } s; void f(void) { -s; }",
    Diagnosis::InvalidOperand,
    ["struct S lvalue <LValueToRValue> struct S", "None"]
);

// ---- 6.3.6 additive operators --------------------------------------------

rejects!(
    add_rejects_two_structures,
    "struct S { int x; } s; void f(void) { s + s; }",
    Diagnosis::InvalidOperand,
    [
        "struct S lvalue <LValueToRValue> struct S",
        "struct S lvalue <LValueToRValue> struct S",
        "None"
    ]
);

rejects!(
    add_rejects_a_floating_index,
    "int *p; void f(void) { p + 1.5; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "double", "None"]
);

rejects!(
    add_rejects_a_pointer_to_void,
    "void *p; void f(void) { p + 1; }",
    Diagnosis::InvalidOperand,
    ["*void lvalue <LValueToRValue> *void", "int", "None"]
);

rejects!(
    add_rejects_a_pointer_to_an_incomplete_type,
    "struct S *p; void f(void) { p + 1; }",
    Diagnosis::InvalidOperand,
    [
        "*struct S (incomplete) lvalue <LValueToRValue> *struct S (incomplete)",
        "int",
        "None"
    ]
);

types!(
    a_pointer_plus_an_integer_is_a_pointer,
    "int *p; void f(void) { p + 1; }",
    ["*int lvalue <LValueToRValue> *int", "int", "*int"]
);

types!(
    an_integer_plus_a_pointer_is_a_pointer,
    "int *p; void f(void) { 1 + p; }",
    ["int", "*int lvalue <LValueToRValue> *int", "*int"]
);

types!(
    an_array_plus_an_integer_is_a_pointer_to_the_element,
    "char a[10]; void f(void) { a + 1; }",
    ["int", "int", "char[10] lvalue <ArrayToPointer> *char", "int", "*char"]
);

// ---- 6.3.4 cast operators --------------------------------------------------

types!(
    a_cast_result_is_always_an_rvalue,
    "int i; void f(void) { (double)i; }",
    ["int lvalue <LValueToRValue> int <IntegerToFloating> double", "double"]
);

types!(
    a_pointer_may_be_cast_to_an_integer,
    "int *p; void f(void) { (int)p; }",
    ["*int lvalue <LValueToRValue> *int <PointerToInteger> int", "int"]
);

types!(
    an_integer_may_be_cast_to_a_pointer,
    "int i; void f(void) { (int *)i; }",
    ["int lvalue <LValueToRValue> int <IntegerToPointer> *int", "*int"]
);

types!(
    a_cast_to_void_discards_the_value,
    "void f(void) { (void)1; }",
    ["int", "void"]
);

rejects!(
    cast_of_a_non_scalar_operand_is_rejected,
    "struct S { int a; } s; void f(void) { (int)s; }",
    Diagnosis::CastToNonScalar,
    ["struct S lvalue <LValueToRValue> struct S", "None"]
);

rejects!(
    cast_to_a_non_scalar_type_is_rejected,
    "struct S { int a; }; void f(void) { (struct S)1; }",
    Diagnosis::CastToNonScalar,
    ["int", "None"]
);

rejects!(
    cast_from_a_pointer_to_a_floating_type_is_rejected,
    "int *p; void f(void) { (double)p; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "None"]
);

rejects!(
    cast_from_a_floating_type_to_a_pointer_is_rejected,
    "double d; void f(void) { (int *)d; }",
    Diagnosis::InvalidOperand,
    ["double lvalue <LValueToRValue> double", "None"]
);

#[test]
fn cast_to_a_non_scalar_type_is_reported_once() {
    let unit = Unit::compile("struct S { int a; }; enum E { A = (struct S)1 };");
    assert!(unit.parsed(), "cc1 failed to parse:\n{}", unit.render());
    let got: Vec<_> = unit.diagnosis().iter().map(|diag| diag.inner).collect();
    assert!(
        matches!(got.as_slice(), [Diagnosis::CastToNonScalar]),
        "expected exactly one CastToNonScalar, got {got:?}:\n{}",
        unit.render()
    );
}

// ---- 6.4 constant expressions --------------------------------------------

types!(
    a_constant_expression_mirrors_its_operand,
    "int a[2 + 3];",
    ["int", "int", "int", "int"]
);

// ---- 6.3.6 additive operators: subtraction --------------------------------

types!(
    a_pointer_minus_an_integer_is_a_pointer,
    "int *p; void f(void) { p - 1; }",
    ["*int lvalue <LValueToRValue> *int", "int", "*int"]
);

types!(
    two_compatible_pointers_subtract_to_ptrdiff_t,
    "int *p, *q; void f(void) { p - q; }",
    [
        "*int lvalue <LValueToRValue> *int",
        "*int lvalue <LValueToRValue> *int",
        "int"
    ]
);

types!(
    pointer_subtraction_works_for_any_compatible_object_type,
    "char *p, *q; void f(void) { p - q; }",
    [
        "*char lvalue <LValueToRValue> *char",
        "*char lvalue <LValueToRValue> *char",
        "int"
    ]
);

rejects!(
    an_integer_minus_a_pointer_is_rejected,
    "int i; int *p; void f(void) { i - p; }",
    Diagnosis::InvalidOperand,
    [
        "int lvalue <LValueToRValue> int",
        "*int lvalue <LValueToRValue> *int",
        "None"
    ]
);

rejects!(
    subtracting_pointers_to_an_incomplete_type_is_rejected,
    "struct S *p, *q; void f(void) { p - q; }",
    Diagnosis::InvalidOperand,
    [
        "*struct S (incomplete) lvalue <LValueToRValue> *struct S (incomplete)",
        "*struct S (incomplete) lvalue <LValueToRValue> *struct S (incomplete)",
        "None"
    ]
);

rejects!(
    subtracting_a_floating_index_is_rejected,
    "int *p; void f(void) { p - 1.5; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "double", "None"]
);

// ---- 6.3.16.1 simple assignment --------------------------------------------

types!(
    assigning_a_constant_keeps_the_lvalues_type,
    "int x; void f(void) { x = 1; }",
    ["int lvalue", "int", "int"]
);

types!(
    assignment_converts_the_right_operand_to_the_left_operands_type,
    "int x; void f(void) { x = 3.5; }",
    ["int lvalue", "double <FloatingToInteger> int", "int"]
);

types!(
    zero_is_a_null_pointer_constant,
    "int *p; void f(void) { p = 0; }",
    ["*int lvalue", "int <NullPointer> *int", "*int"]
);

rejects!(
    assigning_an_incompatible_pointer_is_rejected,
    "char *p; int *q; void f(void) { q = p; }",
    Diagnosis::IncompatibleAssignementTypes,
    ["*int lvalue", "*char lvalue <LValueToRValue> *char", "None"]
);

rejects!(
    assigning_away_const_through_a_pointer_is_rejected,
    "const char *p; char *q; void f(void) { q = p; }",
    Diagnosis::DiscardedQualifiers(_),
    [
        "*char lvalue",
        "*const char lvalue <LValueToRValue> *const char",
        "None"
    ]
);

types!(
    assigning_a_pointer_that_gains_const_is_accepted,
    "char *p; const char *q; void f(void) { q = p; }",
    [
        "*const char lvalue",
        "*char lvalue <LValueToRValue> *char <PointerConversion> *const char",
        "*const char"
    ]
);

types!(
    a_compatible_structure_may_be_assigned,
    "struct S { int a; } x, y; void f(void) { x = y; }",
    [
        "struct S lvalue",
        "struct S lvalue <LValueToRValue> struct S",
        "struct S"
    ]
);

rejects!(
    assigning_to_an_rvalue_is_rejected,
    "void f(void) { 1 = 1; }",
    Diagnosis::AssignToRValue,
    ["int", "int", "None"]
);

rejects!(
    assigning_to_a_const_variable_is_rejected,
    "void f(void) { const int x; x = 1; }",
    Diagnosis::ConstAssignement,
    ["const int lvalue", "int", "None"]
);

// ---- 6.5.7 initialization ---------------------------------------------

types!(
    initializing_with_a_constant_keeps_its_type,
    "void f(void) { int x = 1; }",
    ["int"]
);

types!(
    initialization_converts_the_initializer_to_the_declared_type,
    "void f(void) { int x = 3.5; }",
    ["double <FloatingToInteger> int"]
);

types!(
    zero_initializes_a_pointer_as_a_null_pointer_constant,
    "void f(void) { int *p = 0; }",
    ["int <NullPointer> *int"]
);

rejects!(
    initializing_a_pointer_with_a_double_is_rejected,
    "void f(void) { int *p = 3.5; }",
    Diagnosis::IncompatibleAssignementTypes,
    ["double"]
);

rejects!(
    initializing_with_an_incompatible_pointer_is_rejected,
    "void f(void) { char *p; int *q = p; }",
    Diagnosis::IncompatibleAssignementTypes,
    ["*char lvalue"]
);

rejects!(
    initializing_away_const_through_a_pointer_is_rejected,
    "void f(void) { const char *p; char *q = p; }",
    Diagnosis::DiscardedQualifiers(_),
    ["*const char lvalue"]
);

types!(
    initializing_a_pointer_that_gains_const_is_accepted,
    "void f(void) { char *p; const char *q = p; }",
    ["*char lvalue <PointerConversion> *const char"]
);

// ---- resolution failures do not cascade ----------------------------------

rejects!(
    an_undeclared_identifier_has_no_type,
    "void f(void) { x; }",
    Diagnosis::UndeclaredIdentifier(_),
    ["None"]
);

rejects!(
    an_operand_without_a_type_is_reported_once,
    "void f(void) { x + 1; }",
    Diagnosis::UndeclaredIdentifier(_),
    ["None", "int", "None"]
);
