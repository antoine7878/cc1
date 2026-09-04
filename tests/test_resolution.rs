use crate::common::{Ty, Unit, accepted, folded};
use cc1::semantic::{Diagnosis, FunctionDefId, SymbolKind};

macro_rules! renders {
    ($name:ident, $src:expr, $symbol:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let unit = accepted($src);
            assert_eq!(unit.describe($symbol).as_deref(), Some($expected), "{}", $src);
        }
    };
}

folds!(fold_literal, "enum E { A = 3 };", ["Int(3)"]);
folds!(fold_addition, "enum E { A = 1 + 2 };", ["Int(3)"]);
folds!(fold_precedence, "enum E { A = 1 + 2 * 3 };", ["Int(7)"]);
folds!(fold_parentheses, "enum E { A = (1 + 2) * 3 };", ["Int(9)"]);
folds!(fold_division, "enum E { A = 7 / 2 };", ["Int(3)"]);
folds!(fold_remainder, "enum E { A = 7 % 2 };", ["Int(1)"]);
folds!(fold_shift, "enum E { A = 1 << 4 };", ["Int(16)"]);
folds!(
    fold_bitwise,
    "enum E { A = 6 & 3, B = 6 | 3, C = 6 ^ 3 };",
    ["Int(2)", "Int(7)", "Int(5)"]
);
folds!(
    fold_unary,
    "enum E { A = -3, B = +3, C = ~0, D = !5 };",
    ["Int(-3)", "Int(3)", "Int(-1)", "Int(0)"]
);
folds!(
    fold_relational,
    "enum E { A = 1 < 2, B = 1 == 2 };",
    ["Int(1)", "Int(0)"]
);
folds!(
    fold_every_relational_operator,
    "enum E { A = 1 > 2, B = 1 <= 2, C = 2 >= 2, D = 1 != 2 };",
    ["Int(0)", "Int(1)", "Int(1)", "Int(1)"]
);
folds!(
    fold_every_relational_operator_when_it_does_not_hold,
    "enum E { A = 2 < 1, B = 1 >= 2, C = 2 <= 1, D = 1 != 1, E = 1 == 1 };",
    ["Int(0)", "Int(0)", "Int(0)", "Int(0)", "Int(1)"]
);
folds!(fold_logical, "enum E { A = 1 && 0, B = 1 || 0 };", ["Int(0)", "Int(1)"]);
folds!(
    fold_unsigned_operands,
    "enum E { A = 7u / 2u, B = 7u % 2u, C = 6u & 3u };",
    ["UnsignedInt(3)", "UnsignedInt(1)", "UnsignedInt(2)"]
);
folds!(
    fold_long_operands,
    "enum E { A = 1L + 2L, B = -1L };",
    ["Long(3)", "Long(-1)"]
);
folds!(
    fold_ternary,
    "enum E { A = 1 ? 2 : 3, B = 0 ? 2 : 3 };",
    ["Int(2)", "Int(3)"]
);
// 6.3.15: the second and third operands undergo the usual arithmetic conversions, so the
// result carries the common type even though only one branch's value is picked.
folds!(
    fold_ternary_converts_to_common_type,
    "enum E { A = 1 ? 2 : 3u };",
    ["UnsignedInt(2)"]
);
folds!(fold_character_constant, "enum E { A = 'a' };", ["Int(97)"]);
folds!(
    fold_overflow_wraps,
    "enum E { A = 2147483647 + 1 };",
    ["Int(-2147483648)"]
);
folds!(
    fold_variant_reference,
    "enum E { A = 1, B = A + 1 };",
    ["Int(1)", "Int(2)"]
);
// `sizeof` is folded to a constant at typing time (so nested uses see it too), which is why
// both the `sizeof` node and the constant-expression node wrapping it appear here.
folds!(
    fold_sizeof_type,
    "enum E { A = sizeof(int) };",
    ["UnsignedInt(4)", "UnsignedInt(4)"]
);
folds!(
    fold_sizeof_struct,
    "struct S { char a; int b; }; enum E { A = sizeof(struct S) };",
    ["UnsignedInt(8)", "UnsignedInt(8)"]
);
folds!(fold_cast_narrows, "enum E { A = (char)300 };", ["Int(44)"]);
folds!(fold_cast_to_unsigned, "enum E { A = (unsigned char)-1 };", ["Int(255)"]);
folds!(fold_bit_field_width, "struct S { int a : 2 + 1; };", ["Int(3)"]);
folds!(fold_array_size, "int a[2 + 3];", ["Int(5)"]);
folds!(
    fold_logical_short_circuits,
    "enum E { A = 1 || 1 / 0, B = 0 && 1 / 0 };",
    ["Int(1)", "Int(0)"]
);
folds!(fold_cast_of_a_floating_constant, "enum E { A = (int)1.5 };", ["Int(1)"]);
// gcc: `1L << 31` is -2147483648 on i386, where a long is 32 bits, so it fits an enum variant.
folds!(
    fold_long_shift_narrows_to_the_target_width,
    "enum E { A = 1L << 31 };",
    ["Long(-2147483648)"]
);

#[test]
fn fold_sizeof_of_a_pointer_type() {
    assert_eq!(
        folded("enum E { A = sizeof(char *) };"),
        ["UnsignedInt(4)", "UnsignedInt(4)"]
    );
    assert_eq!(
        folded("enum E { A = sizeof(int *) };"),
        ["UnsignedInt(4)", "UnsignedInt(4)"]
    );
}

#[test]
fn a_non_constant_expression_folds_to_nothing() {
    let unit = Unit::compile("int x; enum E { A = x };");
    assert_eq!(unit.folded(), vec!["None".to_string()]);
    assert!(!unit.accepts());
}

#[test]
fn only_constant_expressions_are_folded() {
    let unit = Unit::compile("void f(void) { int a; a = 1 + 2; }");
    assert!(unit.accepts(), "{}", unit.render());
    assert!(unit.folded().is_empty());
}

#[test]
fn an_identifier_binds_to_its_file_scope_declaration() {
    let unit = accepted("int x; void f(void) { x = 1; }");
    assert_eq!(unit.bindings(), vec![("x".to_string(), Some(0))]);
    assert_eq!(unit.symbols()[0].0, "x");
}

#[test]
fn an_identifier_binds_to_the_innermost_declaration() {
    let unit = accepted("int x; void f(void) { int x; x = 1; }");
    assert_eq!(unit.symbols().len(), 3);
    assert_eq!(unit.bindings(), vec![("x".to_string(), Some(2))]);
}

#[test]
fn an_identifier_binds_to_a_parameter() {
    let unit = accepted("void f(int a) { a = 1; }");
    let symbols = unit.symbols();
    assert_eq!((symbols[1].0.as_str(), symbols[1].1.as_str()), ("a", "parameter"));
    assert_eq!(unit.symbol_ty_tree("a"), Ty::Int);
    assert_eq!(unit.bindings(), vec![("a".to_string(), Some(1))]);
}

#[test]
fn an_identifier_binds_to_an_enumeration_variant() {
    let unit = accepted("enum E { A }; int f(void) { return A; }");
    assert_eq!(unit.symbols()[0].1, "variant");
    assert_eq!(unit.bindings(), vec![("A".to_string(), Some(0))]);
}

#[test]
fn an_undeclared_identifier_binds_to_nothing() {
    let unit = Unit::compile("void f(void) { x = 1; }");
    assert_eq!(unit.bindings(), vec![("x".to_string(), None)]);
    assert!(!unit.accepts());
}

#[test]
fn each_occurrence_is_bound_separately() {
    let unit = accepted("int x; int y; void f(void) { x = y; y = x; }");
    assert_eq!(
        unit.bindings(),
        vec![
            ("x".to_string(), Some(0)),
            ("y".to_string(), Some(1)),
            ("y".to_string(), Some(1)),
            ("x".to_string(), Some(0)),
        ]
    );
}

tree!(describe_int, "int x;", "x", Ty::Int);
tree!(describe_implicit_int, "static x;", "x", Ty::Int);
tree!(describe_char, "char x;", "x", Ty::Char);
tree!(describe_signed_char, "signed char x;", "x", Ty::SChar);
tree!(describe_unsigned_char, "unsigned char x;", "x", Ty::UChar);
tree!(describe_short, "short x;", "x", Ty::Short);
tree!(describe_short_int, "short int x;", "x", Ty::Short);
tree!(describe_unsigned, "unsigned x;", "x", Ty::UInt);
tree!(describe_long, "long x;", "x", Ty::Long);
tree!(describe_unsigned_long, "unsigned long int x;", "x", Ty::ULong);
tree!(describe_float, "float x;", "x", Ty::Float);
tree!(describe_double, "double x;", "x", Ty::Double);
tree!(describe_long_double, "long double x;", "x", Ty::LDouble);
tree!(describe_const, "const int x;", "x", Ty::konst(Ty::Int));
tree!(describe_volatile, "volatile int x;", "x", Ty::vol(Ty::Int));
tree!(
    describe_const_volatile,
    "const volatile int x;",
    "x",
    Ty::konst(Ty::vol(Ty::Int))
);
tree!(describe_pointer, "char *p;", "p", Ty::ptr(Ty::Char));
tree!(describe_pointer_to_pointer, "int **p;", "p", Ty::ptr(Ty::ptr(Ty::Int)));
tree!(
    describe_pointer_to_const,
    "const int *p;",
    "p",
    Ty::ptr(Ty::konst(Ty::Int))
);
tree!(
    describe_const_pointer,
    "int *const p;",
    "p",
    Ty::konst(Ty::ptr(Ty::Int))
);
tree!(describe_struct, "struct S { int a; } s;", "s", Ty::strukt("S"));
tree!(describe_union, "union U { int a; } u;", "u", Ty::union("U"));
tree!(describe_enum, "enum E { A } e;", "e", Ty::enom("E"));
tree!(
    describe_incomplete_struct,
    "struct S; struct S *p;",
    "p",
    Ty::ptr(Ty::strukt_incomplete("S"))
);
tree!(describe_typedef_target, "typedef unsigned int T;", "T", Ty::UInt);
tree!(
    describe_through_typedef,
    "typedef char *S; S s;",
    "s",
    Ty::ptr(Ty::Char)
);
tree!(
    describe_qualified_typedef,
    "typedef int T; const T x;",
    "x",
    Ty::konst(Ty::Int)
);
tree!(describe_member, "struct S { double a; };", "a", Ty::Double);
tree!(
    describe_anonymous_struct_typedef,
    "typedef struct { int a; } T; T x;",
    "x",
    Ty::anon_struct()
);
tree!(describe_parameter, "void f(char *s) { }", "s", Ty::ptr(Ty::Char));
tree!(
    describe_function_returns,
    "long f(void) { return 0; }",
    "f",
    Ty::func0(Ty::Long)
);
tree!(
    describe_function_returning_pointer,
    "int *f(void) { return 0; }",
    "f",
    Ty::func0(Ty::ptr(Ty::Int))
);
tree!(
    describe_function_without_prototype,
    "int f() { return 0; }",
    "f",
    Ty::noproto(Ty::Int)
);
tree!(
    describe_function_parameters,
    "void f(int a, char *s) { }",
    "f",
    Ty::func(Ty::Void, [Ty::Int, Ty::ptr(Ty::Char)])
);
tree!(
    describe_variadic_function,
    "int f(char *s, ...) { return 0; }",
    "f",
    Ty::func_variadic(Ty::Int, [Ty::ptr(Ty::Char)])
);
tree!(
    describe_pointer_to_function,
    "int (*p)(void);",
    "p",
    Ty::ptr(Ty::func0(Ty::Int))
);
tree!(
    describe_array_of_pointer_to_function,
    "int (*p[3])(void);",
    "p",
    Ty::arr(Ty::ptr(Ty::func0(Ty::Int)), 3)
);
tree!(describe_array, "int a[3];", "a", Ty::arr(Ty::Int, 3));
tree!(
    describe_array_of_array,
    "int a[3][5];",
    "a",
    Ty::arr(Ty::arr(Ty::Int, 5), 3)
);
tree!(
    describe_array_of_array_of_array,
    "int a[3][5][7];",
    "a",
    Ty::arr(Ty::arr(Ty::arr(Ty::Int, 7), 5), 3)
);
tree!(describe_incomplete_array, "extern int a[];", "a", Ty::flex(Ty::Int));
tree!(
    describe_incomplete_array_of_array,
    "extern int a[][5];",
    "a",
    Ty::flex(Ty::arr(Ty::Int, 5))
);
tree!(describe_tentative_array_completed, "int a[];", "a", Ty::arr(Ty::Int, 1));
tree!(
    describe_tentative_array_of_array_completed,
    "int a[][5];",
    "a",
    Ty::arr(Ty::arr(Ty::Int, 5), 1)
);
tree!(
    describe_array_of_pointer,
    "int *a[3];",
    "a",
    Ty::arr(Ty::ptr(Ty::Int), 3)
);
tree!(
    describe_pointer_to_array,
    "int (*p)[3];",
    "p",
    Ty::ptr(Ty::arr(Ty::Int, 3))
);
tree!(
    describe_pointer_to_array_of_array,
    "int (*p)[3][5];",
    "p",
    Ty::ptr(Ty::arr(Ty::arr(Ty::Int, 5), 3))
);
tree!(describe_array_parameter, "void f(int a[3]) { }", "a", Ty::ptr(Ty::Int));
tree!(
    describe_function_parameter,
    "void f(int g(void)) { }",
    "g",
    Ty::ptr(Ty::func0(Ty::Int))
);
tree!(
    describe_adjusted_parameter_types,
    "void f(int a[3], int g(void)) { }",
    "f",
    Ty::func(Ty::Void, [Ty::ptr(Ty::Int), Ty::ptr(Ty::func0(Ty::Int))])
);
tree!(
    describe_old_style_array_parameter,
    "int f(a) int a[3]; { return 0; }",
    "a",
    Ty::ptr(Ty::Int)
);
tree!(
    describe_qualified_parameter,
    "void f(const int a) { }",
    "a",
    Ty::konst(Ty::Int)
);
tree!(
    describe_unqualified_parameter_type,
    "void f(const int a) { }",
    "f",
    Ty::func(Ty::Void, [Ty::Int])
);
tree!(
    describe_old_style_function,
    "int f(a, b) int a; char b; { return a; }",
    "f",
    Ty::noproto(Ty::Int)
);

renders!(render_int, "int x;", "x", "int");
renders!(render_const_int, "const int x;", "x", "const int");
renders!(render_pointer, "char *p;", "p", "char *");
renders!(render_array, "int a[3];", "a", "int[3]");
renders!(
    render_function_type,
    "void f(int a, char *s) { }",
    "f",
    "void(int, char *)"
);

#[test]
fn a_prototype_and_its_definition_declare_one_function() {
    let unit = accepted("int f(int a); int f(int a) { return a; }");
    let functions: Vec<_> = unit
        .symbols()
        .into_iter()
        .filter(|(_, kind, _)| kind == "function")
        .map(|(name, kind, _)| (name, kind))
        .collect();
    assert_eq!(functions, [("f".to_string(), "function".to_string())]);
    assert_eq!(unit.symbol_ty_tree("f"), Ty::func(Ty::Int, [Ty::Int]));
}

#[test]
fn identical_function_types_share_one_interned_type() {
    let unit = accepted("int f(int a); int g(int b); int h(char c);");
    let types: Vec<_> = unit
        .ctx
        .sema
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function)
        .map(|symbol| symbol.ty.expect("function type").id)
        .collect();
    assert_eq!(types[0], types[1]);
    assert_ne!(types[0], types[2]);
}

#[test]
fn a_function_definition_records_its_parameters_in_order() {
    let unit = accepted("int f(int a, char b) { return a; }");
    let names: Vec<_> = unit
        .ctx
        .sema
        .functions
        .iter()
        .map(|def| {
            let parameters: Vec<_> = def
                .parameters
                .iter()
                .map(|&id| id.resolve(&unit.ctx).name.id.resolve(&unit.ctx).clone())
                .collect();
            (
                def.sym.resolve(&unit.ctx).name.id.resolve(&unit.ctx).clone(),
                parameters,
                def.is_complete,
            )
        })
        .collect();
    assert_eq!(names, [("f".to_string(), vec!["a".to_string(), "b".to_string()], true)]);
}

#[test]
fn a_function_definition_takes_its_type_from_its_symbol() {
    let unit = accepted("int f(int a, char b) { return a; }");
    let id = FunctionDefId::from(0);
    let ty = unit
        .ctx
        .sema
        .functions
        .ty(id, &unit.ctx.sema.symbols)
        .expect("function type");
    assert_eq!(unit.ty_tree(ty), Ty::func(Ty::Int, [Ty::Int, Ty::Char]));
}

#[test]
fn an_old_style_definition_records_its_parameters_in_declarator_order() {
    let unit = accepted("int f(a, b) char b; { return a; }");
    let def = unit.ctx.sema.functions.iter().next().expect("function definition");
    let parameters: Vec<_> = def
        .parameters
        .iter()
        .map(|&id| id.resolve(&unit.ctx).name.id.resolve(&unit.ctx).clone())
        .collect();
    assert_eq!(parameters, ["a", "b"]);
}

#[test]
fn symbols_are_recorded_in_declaration_order_with_their_kind() {
    let unit = accepted("typedef int T; struct S { int a; } s; void f(int p) { int l; }");
    let kinds: Vec<_> = unit.symbols().into_iter().map(|(name, kind, _)| (name, kind)).collect();
    assert_eq!(
        kinds,
        vec![
            ("T".to_string(), "typedef".to_string()),
            ("a".to_string(), "member".to_string()),
            ("s".to_string(), "variable".to_string()),
            ("f".to_string(), "function".to_string()),
            ("p".to_string(), "parameter".to_string()),
            ("l".to_string(), "variable".to_string()),
        ]
    );
}

// ---- 6.4 a failed constant expression names why it failed ----------------

recover!(
    array_size_is_not_constant,
    "int x; int a[x];",
    [Diagnosis::NonConstantExpression],
    &[]
);

// 6.5.4.2 The expression delimited by [ and ] (which specifies the size of an array) shall be an
// integral constant expression that has a value greater than zero.
// gcc: `int a[-1];` is "declared as an array with a negative size", `int a[0];` is rejected under
// -pedantic-errors as a zero-length array extension.
recover!(
    array_size_is_negative,
    "int a[-1];",
    [Diagnosis::NegativeArraySize],
    &[]
);

recover!(
    array_size_folds_to_a_negative_value,
    "int a[1 - 2];",
    [Diagnosis::NegativeArraySize],
    &[]
);

recover!(array_size_is_zero, "int a[0];", [Diagnosis::ZeroArraySize], &[]);

recover!(
    a_member_array_size_is_zero,
    "struct S { int a[0]; };",
    [Diagnosis::ZeroArraySize],
    &[]
);

recover!(
    an_inner_array_size_is_zero,
    "int a[1][0];",
    [Diagnosis::ZeroArraySize],
    &[]
);

// An abstract declarator carries the same constraint; the size it could not take leaves the array
// incomplete, which is what the sizeof then reports.
recover!(
    an_abstract_array_size_is_zero,
    "enum E { A = sizeof(int[0]) };",
    [Diagnosis::ZeroArraySize, Diagnosis::SizeofIncomplete(_)],
    &[]
);

// A rejected size must leave the array incomplete rather than a length of 18446744073709551615,
// whose layout overflows.
recover!(
    a_rejected_array_size_leaves_no_layout_to_compute,
    "int a[-1]; int b = sizeof(a);",
    [Diagnosis::NegativeArraySize, Diagnosis::SizeofIncomplete(_)],
    &[]
);

recover!(
    array_size_is_not_an_integer,
    "int a[1.5];",
    [Diagnosis::NonIntArraySize],
    &[]
);

// 6.4 A constant expression shall not contain assignment, increment, decrement,
// function-call, or comma operators, except when they are contained within the operand of a
// sizeof operator: the size of an object is a constant known at translation time.
value!(
    sizeof_of_an_expression_is_a_constant_expression,
    "int i; int a[3]; struct S { char c[7]; } s; enum E { A = sizeof(i), B = sizeof(a), C = sizeof(s) };",
    &[("A", "4"), ("B", "12"), ("C", "7")]
);

recover!(
    sizeof_of_an_incomplete_tag,
    "struct S; enum E { A = sizeof(struct S) };",
    [Diagnosis::SizeofIncomplete(_)],
    &[]
);

recover!(
    sizeof_of_void,
    "enum E { A = sizeof(void) };",
    [Diagnosis::SizeofVoid],
    &[]
);

recover!(
    a_division_by_zero_is_not_constant,
    "enum E { A = 1 / 0 };",
    [Diagnosis::DivisionByZero],
    &[]
);

recover!(
    a_modulo_by_zero_is_not_constant,
    "enum E { A = 1 % 0 };",
    [Diagnosis::ModuloByZero],
    &[]
);

// 6.3.5 The result shall be representable in the type of the operands.
recover!(
    a_quotient_outside_the_type_of_its_operands,
    "enum E { A = (-2147483647 - 1) / -1 };",
    [Diagnosis::ConstantOverflow],
    &[]
);

// 6.4 A constant expression shall not contain assignment, increment, decrement, function-call,
// or comma operators, except when they are contained within the operand of a sizeof operator.
recover!(
    an_assignment_is_not_constant,
    "int x; enum E { A = (x = 1) };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_comma_operator_is_not_constant,
    "enum E { A = (1, 2) };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_function_call_is_not_constant,
    "int f(void); enum E { A = f() };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_subscript_is_not_constant,
    "int a[2]; enum E { A = a[0] };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_string_literal_subscript_is_not_constant,
    "enum E { A = \"abc\"[0] };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_member_access_is_not_constant,
    "struct S { int x; } s; enum E { A = s.x };",
    [Diagnosis::NonConstantExpression],
    &[]
);

// 6.4 ... and floating constants that are the immediate operands of casts: a cast of a folded
// floating expression is not one.
recover!(
    a_cast_of_a_floating_expression_is_not_constant,
    "enum E { A = (int)(1.5 + 1) };",
    [Diagnosis::NonConstantExpression],
    &[]
);

// 6.4 A constant expression shall not contain increment or decrement operators.
recover!(
    an_increment_is_not_constant,
    "int x; enum E { A = ++x };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    a_decrement_is_not_constant,
    "int x; enum E { A = x-- };",
    [Diagnosis::NonConstantExpression],
    &[]
);

// 6.4 An integral constant expression shall have integral type: an address is not one.
recover!(
    an_address_is_not_an_integral_constant,
    "int x; enum E { A = &x };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    an_indirection_is_not_constant,
    "int *p; enum E { A = *p };",
    [Diagnosis::NonConstantExpression],
    &[]
);

recover!(
    sizeof_of_a_bit_field,
    "struct S { int a : 3; } s; enum E { A = sizeof s.a };",
    [Diagnosis::SizeofBitfield],
    &[]
);

recover!(
    cast_to_a_non_scalar_type,
    "struct S { int a; }; enum E { A = (struct S)1 };",
    [Diagnosis::CastToNonScalar],
    &[]
);
