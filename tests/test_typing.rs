use crate::common::{Ty, Unit, ints, lv, none, rv};
use cc1::semantic::CastKind::*;
use cc1::semantic::Diagnosis;

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
    Diagnosis::InvalidUnary(_),
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

// ---- 6.3.2.2 function calls ----------------------------------------------

shaped!(
    a_call_has_the_return_type_of_the_function_and_is_an_rvalue,
    "int g(void); void f(void) { g(); }",
    vec![
        rv(Ty::func0(Ty::Int)).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::Int))),
        rv(Ty::Int),
    ]
);

shaped!(
    a_call_to_a_function_returning_void_has_type_void,
    "void g(void); void f(void) { g(); }",
    vec![
        rv(Ty::func0(Ty::Void)).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::Void))),
        rv(Ty::Void),
    ]
);

shaped!(
    a_call_returning_a_structure_is_an_rvalue,
    "struct S { int a; }; struct S g(void); void f(void) { g(); }",
    vec![
        rv(Ty::func0(Ty::strukt("S"))).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::strukt("S")))),
        rv(Ty::strukt("S")),
    ]
);

shaped!(
    an_argument_is_converted_as_if_by_assignment_to_its_parameter,
    "int g(char, float); void f(void) { g(1, 2); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::Char, Ty::Float]))
            .then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Char, Ty::Float]))),
        rv(Ty::Int).then(IntegerConversion, Ty::Char),
        rv(Ty::Int).then(IntegerToFloating, Ty::Float),
        rv(Ty::Int),
    ]
);

shaped!(
    an_array_argument_becomes_a_pointer_to_its_first_element,
    "int g(char *); char a[4]; void f(void) { g(a); }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))
            .then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))),
        lv(Ty::arr(Ty::Char, 4)).then(ArrayToPointer, Ty::ptr(Ty::Char)),
        rv(Ty::Int),
    ]
);

shaped!(
    a_structure_argument_is_passed_by_value,
    "struct S { int a; }; int g(struct S); struct S s; void f(void) { g(s); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::strukt("S")])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::strukt("S")]))),
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        rv(Ty::Int),
    ]
);

shaped!(
    an_argument_pointer_may_gain_the_qualifiers_of_its_parameter,
    "int g(const char *); char *p; void f(void) { g(p); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::ptr(Ty::konst(Ty::Char))])).then(
            FunctionToPointer,
            Ty::ptr(Ty::func(Ty::Int, [Ty::ptr(Ty::konst(Ty::Char))])),
        ),
        lv(Ty::ptr(Ty::Char))
            .then(LValueToRValue, Ty::ptr(Ty::Char))
            .then(PointerConversion, Ty::ptr(Ty::konst(Ty::Char))),
        rv(Ty::Int),
    ]
);

shaped!(
    a_null_pointer_constant_may_be_passed_to_a_pointer_parameter,
    "int g(char *); void f(void) { g(0); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))
            .then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Char)),
        rv(Ty::Int),
    ]
);

// 6.5.4.3 each parameter declared with qualified type is taken as having the unqualified
// version of its declared type
shaped!(
    a_parameter_qualifier_is_not_part_of_the_function_type,
    "int g(const int); void f(void) { g(1); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    an_enumeration_parameter_takes_an_integer_argument,
    "enum E { A }; int g(enum E); void f(void) { g(A); }",
    vec![
        rv(Ty::func(Ty::Int, [Ty::enom("E")])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::enom("E")]))),
        rv(Ty::Int).then(IntegerConversion, Ty::enom("E")),
        rv(Ty::Int),
    ]
);

shaped!(
    a_variadic_prototype_accepts_arguments_past_its_named_parameters,
    "int g(int, ...); void f(void) { g(1, 2, 3); }",
    vec![
        rv(Ty::func_variadic(Ty::Int, [Ty::Int]))
            .then(FunctionToPointer, Ty::ptr(Ty::func_variadic(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_call_may_go_through_a_pointer_to_function,
    "int (*p)(int); void f(void) { p(1); }",
    vec![
        lv(Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))).then(LValueToRValue, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_call_may_go_through_a_typedefed_function_pointer,
    "typedef int F(int); F *q; void f(void) { q(1); }",
    vec![
        lv(Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))).then(LValueToRValue, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

// 6.3.2.2 The number of arguments shall agree with the number of parameters.
rejects_shaped!(
    calling_a_prototype_with_too_many_arguments_is_rejected,
    "int g(int); void f(void) { g(1, 2); }",
    Diagnosis::TooManyArguments(1, 2),
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    calling_a_prototype_with_too_few_arguments_is_rejected,
    "int g(int, int); void f(void) { g(1); }",
    Diagnosis::TooFewArguments(2, 1),
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int, Ty::Int]))
            .then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int, Ty::Int])),),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    a_prototype_with_no_parameters_takes_no_argument,
    "int g(void); void f(void) { g(1); }",
    Diagnosis::TooManyArguments(0, 1),
    vec![
        rv(Ty::func0(Ty::Int)).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::Int))),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    a_variadic_call_still_needs_an_argument_for_each_named_parameter,
    "int g(int, ...); void f(void) { g(); }",
    Diagnosis::TooFewArguments(1, 0),
    vec![
        rv(Ty::func_variadic(Ty::Int, [Ty::Int]))
            .then(FunctionToPointer, Ty::ptr(Ty::func_variadic(Ty::Int, [Ty::Int]))),
        none(),
    ]
);

rejects_shaped!(
    an_argument_incompatible_with_its_parameter_is_rejected,
    "int g(char *); void f(void) { g(1); }",
    Diagnosis::ArgumentIncompatibleTypes(1, _, _),
    vec![
        rv(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))
            .then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::ptr(Ty::Char)]))),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    a_void_argument_is_rejected,
    "void v(void); int g(int); void f(void) { g(v()); }",
    Diagnosis::ArgumentIncompatibleTypes(1, _, _),
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::func0(Ty::Void)).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::Void))),
        rv(Ty::Void),
        none(),
    ]
);

// 6.3.2.2 The expression that denotes the called function shall have type pointer to function
// returning void or returning an object type other than an array type.
rejects_shaped!(
    calling_an_object_is_rejected,
    "double d; void f(void) { d(); }",
    Diagnosis::CallingNotFunction(_),
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), none()]
);

rejects_shaped!(
    calling_an_array_is_rejected,
    "int arr[3]; void f(void) { arr(); }",
    Diagnosis::CallingNotFunction(_),
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        none(),
    ]
);

rejects_shaped!(
    calling_the_result_of_a_call_is_rejected,
    "int g(int); void f(void) { g(1)(2); }",
    Diagnosis::CallingNotFunction(_),
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    calling_a_function_with_an_incomplete_return_type_is_rejected,
    "struct S; struct S g(void); void f(void) { g(); }",
    Diagnosis::CallingIncompleteReturn(_),
    vec![
        rv(Ty::func0(Ty::strukt_incomplete("S")))
            .then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::strukt_incomplete("S")))),
        none(),
    ]
);

shaped!(
    calling_a_function_returning_a_completed_type_is_accepted,
    "struct S { int x; }; struct S g(void); void f(void) { g(); }",
    vec![
        rv(Ty::func0(Ty::strukt("S"))).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::strukt("S")))),
        rv(Ty::strukt("S")),
    ]
);

shaped!(
    the_result_of_a_call_is_an_unqualified_rvalue,
    "const int g(void); void f(void) { g(); }",
    vec![
        rv(Ty::func0(Ty::konst(Ty::Int))).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::konst(Ty::Int)))),
        rv(Ty::Int),
    ]
);

// 6.3.2.2 The default argument promotions are performed on trailing arguments.
shaped!(
    a_trailing_argument_is_promoted,
    "int g(int, ...); void f(void) { char c; float x; g(1, c, x); }",
    vec![
        rv(Ty::func_variadic(Ty::Int, [Ty::Int]))
            .then(FunctionToPointer, Ty::ptr(Ty::func_variadic(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        lv(Ty::Float)
            .then(LValueToRValue, Ty::Float)
            .then(FloatingConversion, Ty::Double),
        rv(Ty::Int),
    ]
);

shaped!(
    a_trailing_array_argument_becomes_a_pointer,
    "int g(int, ...); char a[4]; void f(void) { g(1, a); }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::func_variadic(Ty::Int, [Ty::Int]))
            .then(FunctionToPointer, Ty::ptr(Ty::func_variadic(Ty::Int, [Ty::Int]))),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Char, 4)).then(ArrayToPointer, Ty::ptr(Ty::Char)),
        rv(Ty::Int),
    ]
);

// 6.3.2.2 If the expression that denotes the called function has a type that does not include a
// prototype, the integral promotions are performed on each argument and arguments that have type
// float are promoted to double.
shaped!(
    an_argument_of_a_call_without_a_prototype_is_promoted,
    "int g(); void f(void) { char c; g(c); }",
    vec![
        rv(Ty::noproto(Ty::Int)).then(FunctionToPointer, Ty::ptr(Ty::noproto(Ty::Int))),
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

rejects_shaped!(
    an_argument_without_a_type_does_not_cascade,
    "int g(int); void f(void) { g(x); }",
    Diagnosis::UndeclaredIdentifier(_),
    vec![
        rv(Ty::func(Ty::Int, [Ty::Int])).then(FunctionToPointer, Ty::ptr(Ty::func(Ty::Int, [Ty::Int]))),
        none(),
        none(),
    ]
);

// ---- 6.3.3.2 address and indirection operators ----------------------------

// 6.3.3.2 The result of the unary & operator is a pointer to the object or function
// designated by its operand.
shaped!(
    the_address_of_an_object_is_a_pointer_to_it,
    "int i; void f(void) { &i; }",
    vec![lv(Ty::Int), rv(Ty::ptr(Ty::Int))]
);

// 6.2.2.1 an lvalue operand of the unary & operator is not converted to the value it
// designates, and an array operand does not become a pointer to its first element.
shaped!(
    the_address_of_an_array_is_a_pointer_to_the_array,
    "int a[3]; void f(void) { &a; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)),
        rv(Ty::ptr(Ty::arr(Ty::Int, 3))),
    ]
);

// 6.3.3.2 The operand shall be either a function designator or an lvalue.
shaped!(
    the_address_of_a_function_is_a_pointer_to_function,
    "int g(void); void f(void) { &g; }",
    vec![rv(Ty::func0(Ty::Int)), rv(Ty::ptr(Ty::func0(Ty::Int)))]
);

// 6.3.3.2 If the operand has type "type", the result has type "pointer to type": the
// qualifiers of the designated object are those of the pointed-to type.
shaped!(
    the_address_of_a_const_object_points_to_a_const_type,
    "void f(void) { const int c; &c; }",
    vec![lv(Ty::konst(Ty::Int)), rv(Ty::ptr(Ty::konst(Ty::Int)))]
);

shaped!(
    the_address_of_a_volatile_object_points_to_a_volatile_type,
    "void f(void) { volatile int v; &v; }",
    vec![lv(Ty::vol(Ty::Int)), rv(Ty::ptr(Ty::vol(Ty::Int)))]
);

// 6.3.3.2 the result of the unary & operator is not itself an lvalue.
rejects_shaped!(
    assigning_to_an_address_is_rejected,
    "int i, *p; void f(void) { &i = p; }",
    Diagnosis::AssignToRValue,
    vec![lv(Ty::Int), rv(Ty::ptr(Ty::Int)), lv(Ty::ptr(Ty::Int)), none()]
);

// 6.3.3.2 The operand shall be either a function designator or an lvalue.
rejects_shaped!(
    the_address_of_a_constant_is_rejected,
    "void f(void) { &1; }",
    Diagnosis::RValueAddress(_),
    vec![rv(Ty::Int), none()]
);

rejects_shaped!(
    the_address_of_an_enumeration_constant_is_rejected,
    "enum E { A }; void f(void) { &A; }",
    Diagnosis::RValueAddress(_),
    vec![rv(Ty::Int), none()]
);

// 6.3.3.2 an lvalue that designates a member of a structure or an element of an array is
// still an lvalue: its address may be taken.
shaped!(
    the_address_of_a_member_is_a_pointer_to_it,
    "struct S { int x; } s; void f(void) { &s.x; }",
    vec![lv(Ty::strukt("S")), lv(Ty::Int), rv(Ty::ptr(Ty::Int))]
);

shaped!(
    the_address_of_an_array_element_is_a_pointer_to_it,
    "int a[3]; void f(void) { &a[0]; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        lv(Ty::Int),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    the_address_of_a_string_literal_is_a_pointer_to_the_array,
    "void f(void) { &\"ab\"; }",
    vec![lv(Ty::arr(Ty::Char, 3)), rv(Ty::ptr(Ty::arr(Ty::Char, 3)))]
);

// 6.3.3.2 If the operand is the result of a unary * operator, neither that operator nor the &
// operator is evaluated and the result is as if both were omitted.
shaped!(
    the_address_of_a_dereference_is_the_pointer_itself,
    "int *p; void f(void) { &*p; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        lv(Ty::Int),
        rv(Ty::ptr(Ty::Int)),
    ]
);

// 6.3.3.2 The operand shall be an lvalue that designates an object that is not a bit-field
// and is not declared with the register storage-class specifier.
reject!(
    the_address_of_a_bit_field_is_rejected,
    "struct S { int x : 3; } s; void f(void) { &s.x; }"
);

reject!(
    the_address_of_a_bit_field_reached_through_a_pointer_is_rejected,
    "struct S { int x : 3; } *p; void f(void) { &p->x; }"
);

reject!(
    the_address_of_a_bit_field_of_a_union_is_rejected,
    "union U { int x : 3; } u; void f(void) { &u.x; }"
);

// 6.3.3.2 only a bit-field member is excluded: a plain member of the same structure keeps an
// address, and so does the structure itself.
shaped!(
    the_address_of_a_plain_member_beside_a_bit_field_is_accepted,
    "struct S { int x : 3; int y; } s; void f(void) { &s.y; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::strukt("S")),
        lv(Ty::Int),
        rv(Ty::ptr(Ty::Int)),
    ]
);

reject!(
    the_address_of_a_register_object_is_rejected,
    "void f(void) { register int i; &i; }"
);

// 6.3.3.2 The operand of the unary * operator shall have pointer type.
reject!(indirection_on_an_integer_is_rejected, "void f(void) { int i; *i; }");

reject!(
    indirection_on_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { *s; }"
);

// 6.3.3.2 If the operand points to an object, the result is an lvalue designating the object.
shaped!(
    indirection_through_a_pointer_designates_an_object,
    "int *p; void f(void) { *p; }",
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), lv(Ty::Int)]
);

accept!(a_dereferenced_pointer_is_assignable, "int *p; void f(void) { *p = 1; }");

// 6.2.2.1 the operand of unary * is converted: an array becomes a pointer to its first
// element, so *a designates the first element of a.
accept!(
    indirection_on_an_array_designates_its_first_element,
    "int a[3]; void f(void) { *a = 1; }"
);

// 6.2.2.1 a function designator is converted to a pointer to function, so *g designates g.
accept!(
    indirection_on_a_function_designator_designates_the_function,
    "int g(void); void f(void) { (*g)(); }"
);

// 6.3.3.2 If the operand points to an object, the result is an lvalue designating the object:
// void is not an object type.
reject!(
    indirection_on_a_pointer_to_void_is_rejected,
    "void *v; void f(void) { *v; }"
);

// ---- 6.3.3.3 unary arithmetic operators ----------------------------------

rejects_shaped!(
    minus_rejects_a_pointer,
    "int *p; void f(void) { -p; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), none()]
);

rejects_shaped!(
    minus_rejects_a_structure,
    "struct S { int x; } s; void f(void) { -s; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

// 6.3.3.3 The result of the unary + operator is the value of its operand. The integral
// promotion is performed on the operand, and the result has the promoted type.
shaped!(
    unary_plus_yields_the_value_of_its_operand,
    "int i; void f(void) { +i; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

shaped!(
    unary_plus_promotes_its_operand,
    "void f(void) { char c; +c; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

// 6.3.3.3 The operand of the unary + or - operator shall have arithmetic type.
rejects_shaped!(
    unary_plus_rejects_a_pointer,
    "int *p; void f(void) { +p; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), none()]
);

rejects_shaped!(
    unary_plus_rejects_a_structure,
    "struct S { int x; } s; void f(void) { +s; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

// 6.3.3.3 The integral promotion is performed on the operand of ~, and the result has the
// promoted type.
shaped!(
    a_complement_yields_the_promoted_type_of_its_operand,
    "int i; void f(void) { ~i; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

shaped!(
    a_complement_promotes_its_operand,
    "void f(void) { char c; ~c; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_complement_of_an_unsigned_operand_stays_unsigned,
    "void f(void) { unsigned u; ~u; }",
    vec![lv(Ty::UInt).then(LValueToRValue, Ty::UInt), rv(Ty::UInt)]
);

// 6.3.3.3 The operand of the ~ operator shall have integral type.
rejects_shaped!(
    a_complement_rejects_a_floating_operand,
    "void f(void) { double d; ~d; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), none()]
);

rejects_shaped!(
    a_complement_rejects_a_pointer,
    "int *p; void f(void) { ~p; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), none()]
);

// 6.3.3.3 The result of the logical negation operator ! is 0 or 1: the result has type int.
shaped!(
    a_logical_negation_has_type_int,
    "int i; void f(void) { !i; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

shaped!(
    a_logical_negation_of_a_floating_operand_has_type_int,
    "void f(void) { double d; !d; }",
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), rv(Ty::Int)]
);

shaped!(
    a_logical_negation_of_a_pointer_has_type_int,
    "int *p; void f(void) { !p; }",
    vec![lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)), rv(Ty::Int)]
);

// 6.3.3.3 the result of ! has type int whatever the operand is: the operand is not promoted.
shaped!(
    a_logical_negation_does_not_promote_its_operand,
    "void f(void) { char c; !c; }",
    vec![lv(Ty::Char).then(LValueToRValue, Ty::Char), rv(Ty::Int)]
);

// 6.3.3.3 The operand of the unary ! operator shall have scalar type.
rejects_shaped!(
    a_logical_negation_rejects_a_structure,
    "struct S { int x; } s; void f(void) { !s; }",
    Diagnosis::InvalidUnary(_),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

// 6.2.2.1 the operand of ! is converted: an array becomes a pointer to its first element and
// a function designator becomes a pointer to function, both of which are scalar.
shaped!(
    a_logical_negation_of_an_array_is_accepted,
    "int a[3]; void f(void) { !a; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

shaped!(
    a_logical_negation_of_a_function_designator_is_accepted,
    "int g(); void f(void) { !g; }",
    vec![
        rv(Ty::noproto(Ty::Int)).then(FunctionToPointer, Ty::ptr(Ty::noproto(Ty::Int))),
        rv(Ty::Int),
    ]
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
    Diagnosis::AssignmentDiscardedQualifiers(_, _),
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
    Diagnosis::ConstAssignment(_),
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
    Diagnosis::InitDiscardedQualifiers(_, _),
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

// ---- 6.3.2.1 array subscripting -------------------------------------------

// 6.3.2.1 The expression E1[E2] is identical (by definition) to (*((E1)+(E2))).
shaped!(
    subscripting_an_array_designates_an_element,
    "int a[3]; void f(void) { a[0]; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        lv(Ty::Int),
    ]
);

shaped!(
    subscripting_a_pointer_designates_the_object_it_points_to,
    "int *p; void f(void) { p[1]; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        lv(Ty::Int),
    ]
);

shaped!(
    a_subscript_may_precede_the_array,
    "int a[3]; void f(void) { 0[a]; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        lv(Ty::Int),
    ]
);

shaped!(
    a_subscripted_element_is_assignable,
    "int a[3]; void f(void) { a[0] = 1; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        lv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    subscripting_an_array_of_arrays_designates_a_row,
    "int a[2][3]; void f(void) { a[0]; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::arr(Ty::Int, 3), 2)).then(ArrayToPointer, Ty::ptr(Ty::arr(Ty::Int, 3))),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)),
    ]
);

rejects_shaped!(
    subscripting_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { s[0]; }",
    Diagnosis::InvalidOperand,
    vec![
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        rv(Ty::Int),
        none(),
    ]
);

// ---- 6.3.2.3 structure and union members ----------------------------------

// 6.2.2.1 Except when it is the operand of the sizeof operator, the unary & operator, the ++
// operator, the -- operator, or the left operand of the . operator or an assignment operator,
// an lvalue that does not have array type is converted to the value stored in the designated
// object (and is no longer an lvalue).

shaped!(
    a_member_of_a_structure_lvalue_is_an_lvalue,
    "struct S { int x; } s; void f(void) { s.x; }",
    vec![lv(Ty::strukt("S")), lv(Ty::Int)]
);

shaped!(
    a_member_reached_through_a_pointer_is_an_lvalue,
    "struct S { int x; } *p; void f(void) { p->x; }",
    vec![
        lv(Ty::ptr(Ty::strukt("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt("S"))),
        lv(Ty::Int),
    ]
);

shaped!(
    a_union_member_has_the_type_of_the_named_member,
    "union U { int x; double y; } u; void f(void) { u.y; }",
    vec![lv(Ty::union("U")), lv(Ty::Double)]
);

// 6.3.2.3 The value is that of the named member, and is an lvalue if the first expression is
// an lvalue.
shaped!(
    a_member_of_an_rvalue_structure_is_an_rvalue,
    "struct S { int x; }; struct S g(void); void f(void) { g().x; }",
    vec![
        rv(Ty::func0(Ty::strukt("S"))).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::strukt("S")))),
        rv(Ty::strukt("S")),
        rv(Ty::Int),
    ]
);

rejects_shaped!(
    assigning_to_a_member_of_an_rvalue_structure_is_rejected,
    "struct S { int x; }; struct S g(void); void f(void) { g().x = 1; }",
    Diagnosis::AssignToRValue,
    vec![
        rv(Ty::func0(Ty::strukt("S"))).then(FunctionToPointer, Ty::ptr(Ty::func0(Ty::strukt("S")))),
        rv(Ty::strukt("S")),
        rv(Ty::Int),
        rv(Ty::Int),
        none(),
    ]
);

// 6.3.2.3 If the first expression is a pointer to a qualified type, the value has the
// so-qualified version of the type of the designated member.
reject!(
    assigning_to_a_member_of_a_const_structure_is_rejected,
    "struct S { int x; }; void f(void) { const struct S s; s.x = 1; }"
);

reject!(
    assigning_to_a_member_through_a_pointer_to_const_is_rejected,
    "struct S { int x; }; void f(void) { const struct S *p; p->x = 1; }"
);

// 6.3.2.3 The first operand of the . operator shall have qualified or unqualified structure or
// union type, and the second operand shall name a member of that type.
rejects_shaped!(
    a_dot_applied_to_a_pointer_is_rejected,
    "struct S { int x; } *p; void f(void) { p.x; }",
    Diagnosis::AccessNotStuctOrUnion(_),
    vec![lv(Ty::ptr(Ty::strukt("S"))), none()]
);

rejects_shaped!(
    a_dot_applied_to_an_enumeration_is_rejected,
    "enum E { A }; enum E e; void f(void) { e.x; }",
    Diagnosis::AccessNotStuctOrUnion(_),
    vec![lv(Ty::enom("E")), none()]
);

rejects_shaped!(
    a_member_that_the_structure_does_not_have_is_rejected,
    "struct S { int x; } s; void f(void) { s.y; }",
    Diagnosis::AccessNotMember(_, _),
    vec![lv(Ty::strukt("S")), none()]
);

// 6.3.2.3 The first operand of the -> operator shall have type pointer to qualified or
// unqualified structure or pointer to qualified or unqualified union.
rejects_shaped!(
    an_arrow_applied_to_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { s->x; }",
    Diagnosis::AccessNotPointer(_),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

reject!(
    a_member_of_an_incomplete_structure_is_rejected,
    "struct S; struct S *p; void f(void) { p->x; }"
);

// 6.3.2.3 If the first expression is a pointer to a qualified type, the value has the
// so-qualified version of the type of the designated member.
shaped!(
    a_member_of_a_const_structure_is_const,
    "struct S { int x; }; void f(void) { const struct S s; s.x; }",
    vec![lv(Ty::konst(Ty::strukt("S"))), lv(Ty::konst(Ty::Int))]
);

shaped!(
    a_member_reached_through_a_pointer_to_const_is_const,
    "struct S { int x; }; void f(void) { const struct S *p; p->x; }",
    vec![
        lv(Ty::ptr(Ty::konst(Ty::strukt("S")))).then(LValueToRValue, Ty::ptr(Ty::konst(Ty::strukt("S")))),
        lv(Ty::konst(Ty::Int)),
    ]
);

shaped!(
    a_member_of_a_volatile_structure_is_volatile,
    "volatile struct S { int x; } s; void f(void) { s.x; }",
    vec![lv(Ty::vol(Ty::strukt("S"))), lv(Ty::vol(Ty::Int))]
);

// ---- 6.3.2.4 postfix increment and decrement operators --------------------

// 6.3.2.4 The result of the postfix ++ operator is the value of the operand.
shaped!(
    post_increment_yields_the_value_of_its_operand,
    "int i; void f(void) { i++; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

shaped!(
    post_decrement_yields_the_value_of_its_operand,
    "double d; void f(void) { d--; }",
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), rv(Ty::Double)]
);

shaped!(
    post_increment_of_a_pointer_is_a_pointer,
    "int *p; void f(void) { p++; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::ptr(Ty::Int)),
    ]
);

// 6.3.2.4 the value of the result has the type of the operand: it is not promoted.
shaped!(
    post_increment_keeps_the_type_of_its_operand,
    "char c; void f(void) { c++; }",
    vec![lv(Ty::Char).then(LValueToRValue, Ty::Char), rv(Ty::Char)]
);

// 6.3.2.4 The operand shall have qualified or unqualified scalar type and shall be a
// modifiable lvalue.
reject!(post_increment_of_an_rvalue_is_rejected, "void f(void) { 1++; }");

reject!(
    post_increment_of_a_const_object_is_rejected,
    "void f(void) { const int i; i++; }"
);

reject!(
    post_increment_of_an_array_is_rejected,
    "int a[3]; void f(void) { a++; }"
);

rejects_shaped!(
    post_increment_of_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { s++; }",
    Diagnosis::BadPostIncDec(_, _),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

// 6.3.2.4 The value of the operand is incremented: see the discussion of additive operators,
// which requires a pointer to an object type (6.3.6).
rejects_shaped!(
    post_increment_of_a_pointer_to_an_incomplete_type_is_rejected,
    "struct S; struct S *p; void f(void) { p++; }",
    Diagnosis::IncompleteType(_),
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        none(),
    ]
);

rejects_shaped!(
    post_increment_of_a_pointer_to_void_is_rejected,
    "void *p; void f(void) { p++; }",
    Diagnosis::IncompleteType(_),
    vec![lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)), none()]
);

// ---- 6.3.3.1 prefix increment and decrement operators ---------------------

shaped!(
    pre_increment_yields_the_value_of_its_operand,
    "int i; void f(void) { ++i; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int)]
);

shaped!(
    pre_decrement_yields_the_value_of_its_operand,
    "double d; void f(void) { --d; }",
    vec![lv(Ty::Double).then(LValueToRValue, Ty::Double), rv(Ty::Double)]
);

shaped!(
    pre_increment_of_a_pointer_is_a_pointer,
    "int *p; void f(void) { ++p; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::ptr(Ty::Int)),
    ]
);

shaped!(
    pre_increment_keeps_the_type_of_its_operand,
    "char c; void f(void) { ++c; }",
    vec![lv(Ty::Char).then(LValueToRValue, Ty::Char), rv(Ty::Char)]
);

// 6.3.3.1 The expression ++E is equivalent to (E += 1): its result is not an lvalue.
rejects_shaped!(
    assigning_to_a_pre_increment_is_rejected,
    "int i; void f(void) { ++i = 1; }",
    Diagnosis::AssignToRValue,
    vec![
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
        none(),
    ]
);

// 6.3.3.1 The operand shall have qualified or unqualified scalar type and shall be a
// modifiable lvalue.
reject!(pre_increment_of_an_rvalue_is_rejected, "void f(void) { ++1; }");

reject!(
    pre_increment_of_a_const_object_is_rejected,
    "void f(void) { const int i; ++i; }"
);

reject!(pre_increment_of_an_array_is_rejected, "int a[3]; void f(void) { ++a; }");

rejects_shaped!(
    pre_increment_of_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { ++s; }",
    Diagnosis::BadPostIncDec(_, _),
    vec![lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")), none()]
);

rejects_shaped!(
    pre_decrement_of_a_pointer_to_an_incomplete_type_is_rejected,
    "struct S; struct S *p; void f(void) { --p; }",
    Diagnosis::IncompleteType(_),
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        none(),
    ]
);

// ---- 6.3.5 multiplicative operators ---------------------------------------

shaped!(multiplying_two_constants_stays_int, "void f(void) { 2 * 3; }", ints(3));

shaped!(
    the_remainder_of_two_integers_is_an_integer,
    "void f(void) { 7 % 2; }",
    ints(3)
);

shaped!(
    multiplication_performs_the_usual_arithmetic_conversions,
    "int i; double d; void f(void) { i * d; }",
    vec![
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToFloating, Ty::Double),
        lv(Ty::Double).then(LValueToRValue, Ty::Double),
        rv(Ty::Double),
    ]
);

// 6.3.5 Each of the operands shall have arithmetic type.
rejects_shaped!(
    multiplying_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { s * 1; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        rv(Ty::Int),
        none(),
    ]
);

rejects_shaped!(
    multiplying_by_a_structure_is_rejected,
    "struct S { int x; } s; void f(void) { 1 * s; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        rv(Ty::Int),
        lv(Ty::strukt("S")).then(LValueToRValue, Ty::strukt("S")),
        none(),
    ]
);

rejects_shaped!(
    dividing_a_pointer_is_rejected,
    "int *p; void f(void) { p / 2; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        none(),
    ]
);

// 6.3.5 The operands of the % operator shall have integral type.
rejects_shaped!(
    a_remainder_with_a_floating_right_operand_is_rejected,
    "void f(void) { 1 % 1.5; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![rv(Ty::Int), rv(Ty::Double), none()]
);

rejects_shaped!(
    a_remainder_with_a_floating_left_operand_is_rejected,
    "void f(void) { 1.5 % 1; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![rv(Ty::Double), rv(Ty::Int), none()]
);

// 6.3.5 In both operations, if the value of the second operand is zero, the behavior is
// undefined: only a constant expression has to be diagnosed (6.4).
shaped!(
    a_floating_division_by_zero_is_accepted,
    "void f(void) { 1 / 0.0; }",
    vec![
        rv(Ty::Int).then(IntegerToFloating, Ty::Double),
        rv(Ty::Double),
        rv(Ty::Double),
    ]
);

// ---- 6.3.7 bitwise shift operators ----------------------------------------

shaped!(a_shift_of_two_constants_is_an_int, "void f(void) { 1 << 2; }", ints(3));

// 6.3.7 the behavior is undefined only when the count is negative or greater than or equal to
// the width in bits of the promoted left operand.
shaped!(
    a_shift_count_below_the_width_of_the_promoted_left_operand_is_accepted,
    "void f(void) { 1 << 5; }",
    ints(3)
);

// 6.3.7 The integral promotions are performed on each of the operands.
shaped!(
    the_left_operand_of_a_shift_is_promoted,
    "char c; void f(void) { c << 1; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    an_enumeration_may_be_shifted,
    "enum E { A }; enum E e; void f(void) { e << 1; }",
    vec![
        lv(Ty::enom("E"))
            .then(LValueToRValue, Ty::enom("E"))
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

// 6.3.7 The type of the result is that of the promoted left operand: the usual arithmetic
// conversions are not performed.
shaped!(
    a_shift_has_the_type_of_its_promoted_left_operand,
    "int i; long l; void f(void) { i << l; }",
    vec![
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        lv(Ty::Long).then(LValueToRValue, Ty::Long),
        rv(Ty::Int),
    ]
);

shaped!(
    the_operands_of_a_shift_are_promoted_independently,
    "unsigned u; void f(void) { u >> 1; }",
    vec![lv(Ty::UInt).then(LValueToRValue, Ty::UInt), rv(Ty::Int), rv(Ty::UInt),]
);

// 6.3.7 Each of the operands shall have integral type.
rejects_shaped!(
    shifting_a_floating_left_operand_is_rejected,
    "void f(void) { 1.5 << 1; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![rv(Ty::Double), rv(Ty::Int), none()]
);

rejects_shaped!(
    shifting_by_a_floating_count_is_rejected,
    "void f(void) { 1 << 1.5; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![rv(Ty::Int), rv(Ty::Double), none()]
);

// 6.3.7 the count is compared to the width in bits of the promoted left operand, not to the
// width of the operand as written.
shaped!(
    a_char_left_operand_is_measured_after_its_promotion,
    "char c; void f(void) { c << 10; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_short_left_operand_is_measured_after_its_promotion,
    "short s; void f(void) { s << 20; }",
    vec![
        lv(Ty::Short)
            .then(LValueToRValue, Ty::Short)
            .then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    a_count_one_below_the_width_of_the_left_operand_is_accepted,
    "int i; void f(void) { i << 31; }",
    vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(Ty::Int), rv(Ty::Int)]
);

// 6.3.7 the width that bounds the count is that of the left operand: the type of the count
// itself does not bound it.
shaped!(
    the_type_of_the_count_does_not_bound_the_shift,
    "int i; void f(void) { i << (short)20; }",
    vec![
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        rv(Ty::Int).then(IntegerConversion, Ty::Short),
        rv(Ty::Short).then(IntegerPromotion, Ty::Int),
        rv(Ty::Int),
    ]
);

// 6.3.7 The integral promotions are performed on each of the operands, whether or not the
// count is a constant.
shaped!(
    a_shift_by_a_variable_count_promotes_its_operands,
    "char c; int n; void f(void) { c << n; }",
    vec![
        lv(Ty::Char)
            .then(LValueToRValue, Ty::Char)
            .then(IntegerPromotion, Ty::Int),
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        rv(Ty::Int),
    ]
);

shaped!(
    an_enumeration_may_be_shifted_by_a_variable_count,
    "enum E { A }; enum E e; int n; void f(void) { e << n; }",
    vec![
        lv(Ty::enom("E"))
            .then(LValueToRValue, Ty::enom("E"))
            .then(IntegerPromotion, Ty::Int),
        lv(Ty::Int).then(LValueToRValue, Ty::Int),
        rv(Ty::Int),
    ]
);

// A rejected shift converts nothing: the operands keep the types they were written with.
rejects_shaped!(
    a_rejected_shift_leaves_its_operands_unpromoted,
    "enum E { A }; enum E e; void f(void) { e << 1.5; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::enom("E")).then(LValueToRValue, Ty::enom("E")),
        rv(Ty::Double),
        none(),
    ]
);

// ---- 6.3.8 relational operators ------------------------------------------

// 6.3.8 both operands are pointers to qualified or unqualified versions of compatible object
// types.
shaped!(
    pointers_to_compatible_object_types_may_be_ordered,
    "int *p; int *q; void f(void) { p < q; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

shaped!(
    ordering_pointers_disregards_the_qualifiers_of_the_pointed_to_type,
    "const int *p; int *q; void f(void) { p < q; }",
    vec![
        lv(Ty::ptr(Ty::konst(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::konst(Ty::Int))),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

// 6.3.8 both operands are pointers to qualified or unqualified versions of compatible incomplete
// types.
shaped!(
    pointers_to_an_incomplete_structure_may_be_ordered,
    "struct S; struct S *p; struct S *q; void f(void) { p >= q; }",
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        lv(Ty::ptr(Ty::strukt_incomplete("S"))).then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S"))),
        rv(Ty::Int),
    ]
);

// 6.1.2.5 The void type comprises an empty set of values. it is an incomplete type that cannot be
// completed.
shaped!(
    pointers_to_void_may_be_ordered,
    "void *p; void *q; void f(void) { p > q; }",
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        rv(Ty::Int),
    ]
);

// 6.1.2.5 Types are partitioned into object types, function types, and incomplete types: a
// function type is neither of the two the constraint admits.
rejects_shaped!(
    pointers_to_functions_may_not_be_ordered,
    "int (*p)(void); int (*q)(void); void f(void) { p < q; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::func0(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::func0(Ty::Int))),
        lv(Ty::ptr(Ty::func0(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::func0(Ty::Int))),
        none(),
    ]
);

rejects_shaped!(
    pointers_to_incompatible_types_may_not_be_ordered,
    "int *p; unsigned *q; void f(void) { p < q; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::UInt)).then(LValueToRValue, Ty::ptr(Ty::UInt)),
        none(),
    ]
);

// 6.3.8 If both of the operands have arithmetic type, the usual arithmetic conversions are
// performed. The result has type int.
shaped!(
    ordering_arithmetic_operands_performs_the_usual_arithmetic_conversions,
    "int i; double d; void f(void) { i < d; }",
    vec![
        lv(Ty::Int)
            .then(LValueToRValue, Ty::Int)
            .then(IntegerToFloating, Ty::Double),
        lv(Ty::Double).then(LValueToRValue, Ty::Double),
        rv(Ty::Int),
    ]
);

// 6.2.2.1 an lvalue that has type "array of type" is converted to an expression that has type
// "pointer to type" that points to the initial element of the array object.
shaped!(
    an_array_operand_of_an_ordering_decays_to_a_pointer,
    "int a[3]; int *p; void f(void) { a < p; }",
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::arr(Ty::Int, 3)).then(ArrayToPointer, Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

// 6.5.4.2 For two array types to be compatible, both shall have compatible element types, and if
// both size specifiers are present, they shall have the same value.
// 6.1.2.5 An array type of unknown size is an incomplete type.
shaped!(
    pointers_to_compatible_incomplete_arrays_may_be_ordered,
    "int (*p)[]; int (*q)[]; void f(void) { p < q; }",
    vec![
        lv(Ty::ptr(Ty::flex(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::flex(Ty::Int))),
        lv(Ty::ptr(Ty::flex(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::flex(Ty::Int))),
        rv(Ty::Int),
    ]
);

// 6.3.8 An array of unknown size is compatible with an array of known size, yet the two pointed
// to types are neither both object types nor both incomplete types.
rejects_shaped!(
    a_pointer_to_an_incomplete_array_may_not_be_ordered_with_one_to_a_complete_array,
    "int (*p)[]; int (*q)[3]; void f(void) { p < q; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        rv(Ty::Int),
        rv(Ty::Int),
        lv(Ty::ptr(Ty::flex(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::flex(Ty::Int))),
        lv(Ty::ptr(Ty::arr(Ty::Int, 3))).then(LValueToRValue, Ty::ptr(Ty::arr(Ty::Int, 3))),
        none(),
    ]
);

// 6.3.8 admits no null pointer constant, unlike 6.3.9 one operand is a pointer and the other is
// a null pointer constant.
rejects_shaped!(
    a_pointer_may_not_be_ordered_with_a_null_pointer_constant,
    "int *p; void f(void) { p > 0; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
        none(),
    ]
);

// 6.3.8 admits no pointer to void against a pointer to an object type, unlike 6.3.9 one operand
// is a pointer to an object or incomplete type and the other is a pointer to a qualified or
// unqualified version of void.
rejects_shaped!(
    a_pointer_to_void_may_not_be_ordered_with_a_pointer_to_an_object,
    "void *v; int *p; void f(void) { v < p; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        none(),
    ]
);

// ---- 6.3.9 equality operators --------------------------------------------

// 6.3.9 one operand is a pointer and the other is a null pointer constant.
shaped!(
    a_pointer_compared_to_a_null_pointer_constant_converts_the_constant,
    "int *p; void f(void) { p == 0; }",
    vec![
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

shaped!(
    a_null_pointer_constant_may_be_the_left_operand,
    "int *p; void f(void) { 0 != p; }",
    vec![
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::Int)),
        lv(Ty::ptr(Ty::Int)).then(LValueToRValue, Ty::ptr(Ty::Int)),
        rv(Ty::Int),
    ]
);

// 6.3.9 puts no restriction on the pointed to type of the pointer compared to a null pointer
// constant.
shaped!(
    a_pointer_to_a_function_may_be_compared_to_a_null_pointer_constant,
    "int (*p)(void); void f(void) { p == 0; }",
    vec![
        lv(Ty::ptr(Ty::func0(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::func0(Ty::Int))),
        rv(Ty::Int).then(NullPointer, Ty::ptr(Ty::func0(Ty::Int))),
        rv(Ty::Int),
    ]
);

// 6.3.9 one of the operands is a pointer to an object or incomplete type and the other has type
// pointer to a qualified or unqualified version of void; the pointer to an object or incomplete
// type is converted to the type of the other operand.
shaped!(
    an_object_pointer_compared_to_a_void_pointer_is_converted_to_void_pointer,
    "void *v; int *p; void f(void) { v == p; }",
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::Int))
            .then(LValueToRValue, Ty::ptr(Ty::Int))
            .then(PointerConversion, Ty::ptr(Ty::Void)),
        rv(Ty::Int),
    ]
);

shaped!(
    an_incomplete_pointer_compared_to_a_void_pointer_is_converted_to_void_pointer,
    "struct S; struct S *p; void *v; void f(void) { p != v; }",
    vec![
        lv(Ty::ptr(Ty::strukt_incomplete("S")))
            .then(LValueToRValue, Ty::ptr(Ty::strukt_incomplete("S")))
            .then(PointerConversion, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        rv(Ty::Int),
    ]
);

shaped!(
    comparing_a_qualified_pointer_to_a_void_pointer_discards_its_qualifiers,
    "void *v; const int *p; void f(void) { v == p; }",
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::konst(Ty::Int)))
            .then(LValueToRValue, Ty::ptr(Ty::konst(Ty::Int)))
            .then(PointerConversion, Ty::ptr(Ty::Void)),
        rv(Ty::Int),
    ]
);

// 6.3.9 admits a pointer to an object or incomplete type against a pointer to void, never a
// pointer to a function type.
rejects_shaped!(
    a_pointer_to_a_function_may_not_be_compared_to_a_void_pointer,
    "void *v; int (*p)(void); void f(void) { v == p; }",
    Diagnosis::InvalidBinaryOperand(_, _),
    vec![
        lv(Ty::ptr(Ty::Void)).then(LValueToRValue, Ty::ptr(Ty::Void)),
        lv(Ty::ptr(Ty::func0(Ty::Int))).then(LValueToRValue, Ty::ptr(Ty::func0(Ty::Int))),
        none(),
    ]
);
