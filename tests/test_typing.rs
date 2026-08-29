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

// 6.1.3.3 An identifier declared as an enumeration constant has type int.
types!(
    an_enumeration_constant_is_an_rvalue,
    "enum E { A }; void f(void) { A; }",
    ["int"]
);

// 6.1.4 the array elements are initialized with the individual characters of the character
// string literal plus a terminating null character.
// gcc: sizeof("ab") == 3
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

// gcc: rejects, `invalid argument type 'char *' to unary expression`
rejects!(
    an_array_operand_becomes_a_pointer_to_its_first_element,
    "char a[10]; void f(void) { -a; }",
    Diagnosis::InvalidOperand,
    ["int", "int", "char[10] lvalue <ArrayToPointer> *char", "None"]
);

// gcc: rejects, `arithmetic on a pointer to the function type`
rejects!(
    a_function_designator_becomes_a_pointer_to_function,
    "int g(); void f(void) { g + 1; }",
    Diagnosis::InvalidOperand,
    ["int() <FunctionToPointer> *(int())", "int", "None"]
);

// ---- 6.2.1.1 integral promotions -----------------------------------------

// gcc: sizeof(-c) == 4
types!(
    a_char_operand_promotes_to_int,
    "char c; void f(void) { -c; }",
    ["char lvalue <LValueToRValue> char <IntegerPromotion> int", "int"]
);

// gcc: sizeof(-s) == 4
types!(
    a_short_operand_promotes_to_int,
    "short s; void f(void) { -s; }",
    ["short lvalue <LValueToRValue> short <IntegerPromotion> int", "int"]
);

// 6.2.1.1 All other arithmetic types are unchanged by the integral promotions.
// gcc: sizeof(-d) == 8
types!(
    a_double_operand_is_left_alone,
    "double d; void f(void) { -d; }",
    ["double lvalue <LValueToRValue> double", "double"]
);

// gcc: sizeof(c + c) == 4
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

// gcc: sizeof(i + d) == 8
types!(
    an_integer_and_a_double_meet_at_double,
    "int i; double d; void f(void) { i + d; }",
    [
        "int lvalue <LValueToRValue> int <IntegerToFloating> double",
        "double lvalue <LValueToRValue> double",
        "double"
    ]
);

// gcc: sizeof(g + i) == 4
types!(
    an_integer_and_a_float_meet_at_float,
    "float g; int i; void f(void) { g + i; }",
    [
        "float lvalue <LValueToRValue> float",
        "int lvalue <LValueToRValue> int <IntegerToFloating> float",
        "float"
    ]
);

// gcc: (u + i) has type unsigned int, so -1 + 0u compares greater than 0
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

// 6.3.3.3 The operand of the unary - operator shall have arithmetic type.
// gcc: rejects, `invalid argument type 'int *' to unary expression`
rejects!(
    minus_rejects_a_pointer,
    "int *p; void f(void) { -p; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "None"]
);

// gcc: rejects, `invalid argument type 'struct S' to unary expression`
rejects!(
    minus_rejects_a_structure,
    "struct S { int x; } s; void f(void) { -s; }",
    Diagnosis::InvalidOperand,
    ["struct S lvalue <LValueToRValue> struct S", "None"]
);

// ---- 6.3.6 additive operators --------------------------------------------

// 6.3.6 either both operands shall have arithmetic type, or one operand shall be a pointer to
// an object type and the other shall have integral type.
// gcc: rejects, `invalid operands to binary expression ('struct S' and 'struct S')`
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

// gcc: rejects, `invalid operands to binary expression ('int *' and 'double')`
rejects!(
    add_rejects_a_floating_index,
    "int *p; void f(void) { p + 1.5; }",
    Diagnosis::InvalidOperand,
    ["*int lvalue <LValueToRValue> *int", "double", "None"]
);

// gcc: rejects, `arithmetic on a pointer to void`
rejects!(
    add_rejects_a_pointer_to_void,
    "void *p; void f(void) { p + 1; }",
    Diagnosis::InvalidOperand,
    ["*void lvalue <LValueToRValue> *void", "int", "None"]
);

// gcc: rejects, `arithmetic on a pointer to an incomplete type 'struct S'`
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

// gcc: accepts, and (p + 1) has type int *
types!(
    a_pointer_plus_an_integer_is_a_pointer,
    "int *p; void f(void) { p + 1; }",
    ["*int lvalue <LValueToRValue> *int", "int", "*int"]
);

// gcc: accepts, and (1 + p) has type int *
types!(
    an_integer_plus_a_pointer_is_a_pointer,
    "int *p; void f(void) { 1 + p; }",
    ["int", "*int lvalue <LValueToRValue> *int", "*int"]
);

// gcc: accepts, and (a + 1) has type char *
types!(
    an_array_plus_an_integer_is_a_pointer_to_the_element,
    "char a[10]; void f(void) { a + 1; }",
    ["int", "int", "char[10] lvalue <ArrayToPointer> *char", "int", "*char"]
);

// ---- 6.4 constant expressions --------------------------------------------

types!(
    a_constant_expression_mirrors_its_operand,
    "int a[2 + 3];",
    ["int", "int", "int", "int"]
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
