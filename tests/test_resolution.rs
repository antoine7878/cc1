mod common;

use cc1::semantic::{Diagnosis, FunctionDefId, SymbolKind};
use common::Unit;

fn folded(src: &str) -> Vec<String> {
    let unit = Unit::compile(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    unit.folded()
}

fn accepted(src: &str) -> Unit {
    let unit = Unit::compile(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    unit
}

macro_rules! folds {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(folded($src), $expected, "{}", $src);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            assert_eq!(folded($src), $expected, "{}", $src);
        }
    };
}

macro_rules! describes {
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
// folds!(fold_division_by_zero, "enum E { A = 1 / 0 };", ["Int(0)"]);
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
folds!(fold_logical, "enum E { A = 1 && 0, B = 1 || 0 };", ["Int(0)", "Int(1)"]);
folds!(
    fold_ternary,
    "enum E { A = 1 ? 2 : 3, B = 0 ? 2 : 3 };",
    ["Int(2)", "Int(3)"]
);
folds!(fold_character_constant, "enum E { A = 'a' };", ["Int(97)"]);
folds!(
    ignore "C90 6.3: undefined behavior, the folded value is not guaranteed",
    fold_overflow_wraps,
    "enum E { A = 2147483647 + 1 };",
    ["Int(-2147483648)"]
);
folds!(
    fold_variant_reference,
    "enum E { A = 1, B = A + 1 };",
    ["Int(1)", "Int(2)"]
);
folds!(fold_sizeof_type, "enum E { A = sizeof(int) };", ["UnsignedInt(4)"]);
folds!(
    fold_sizeof_struct,
    "struct S { char a; int b; }; enum E { A = sizeof(struct S) };",
    ["UnsignedInt(8)"]
);
folds!(fold_cast_narrows, "enum E { A = (char)300 };", ["Int(44)"]);
folds!(fold_cast_to_unsigned, "enum E { A = (unsigned char)-1 };", ["Int(255)"]);
folds!(fold_bit_field_width, "struct S { int a : 2 + 1; };", ["Int(3)"]);
folds!(fold_array_size, "int a[2 + 3];", ["Int(5)"]);

#[test]
fn fold_sizeof_of_a_pointer_type() {
    assert_eq!(folded("enum E { A = sizeof(char *) };"), ["UnsignedInt(4)"]);
    assert_eq!(folded("enum E { A = sizeof(int *) };"), ["UnsignedInt(4)"]);
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
    assert_eq!(
        symbols[1],
        ("a".to_string(), "parameter".to_string(), "int".to_string())
    );
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

describes!(describe_int, "int x;", "x", "int");
describes!(describe_implicit_int, "static x;", "x", "int");
describes!(describe_char, "char x;", "x", "char");
describes!(describe_signed_char, "signed char x;", "x", "signed char");
describes!(describe_unsigned_char, "unsigned char x;", "x", "unsigned char");
describes!(describe_short, "short x;", "x", "short");
describes!(describe_short_int, "short int x;", "x", "short");
describes!(describe_unsigned, "unsigned x;", "x", "unsigned int");
describes!(describe_long, "long x;", "x", "long");
describes!(describe_unsigned_long, "unsigned long int x;", "x", "unsigned long");
describes!(describe_float, "float x;", "x", "float");
describes!(describe_double, "double x;", "x", "double");
describes!(describe_long_double, "long double x;", "x", "long double");
describes!(describe_const, "const int x;", "x", "const int");
describes!(describe_volatile, "volatile int x;", "x", "volatile int");
describes!(
    describe_const_volatile,
    "const volatile int x;",
    "x",
    "const volatile int"
);
describes!(describe_pointer, "char *p;", "p", "*char");
describes!(describe_pointer_to_pointer, "int **p;", "p", "**int");
describes!(describe_pointer_to_const, "const int *p;", "p", "*const int");
describes!(describe_const_pointer, "int *const p;", "p", "const *int");
describes!(describe_struct, "struct S { int a; } s;", "s", "struct S");
describes!(describe_union, "union U { int a; } u;", "u", "union U");
describes!(describe_enum, "enum E { A } e;", "e", "enum E");
describes!(
    describe_incomplete_struct,
    "struct S; struct S *p;",
    "p",
    "*struct S (incomplete)"
);
describes!(describe_typedef_target, "typedef unsigned int T;", "T", "unsigned int");
describes!(describe_through_typedef, "typedef char *S; S s;", "s", "*char");
describes!(
    describe_qualified_typedef,
    "typedef int T; const T x;",
    "x",
    "const int"
);
describes!(describe_member, "struct S { double a; };", "a", "double");
describes!(
    describe_anonymous_struct_typedef,
    "typedef struct { int a; } T; T x;",
    "x",
    "struct <anonymous>"
);
describes!(describe_parameter, "void f(char *s) { }", "s", "*char");
describes!(
    describe_function_returns,
    "long f(void) { return 0; }",
    "f",
    "long(void)"
);
describes!(
    describe_function_returning_pointer,
    "int *f(void) { return 0; }",
    "f",
    "*int(void)"
);
describes!(
    describe_function_without_prototype,
    "int f() { return 0; }",
    "f",
    "int()"
);
describes!(
    describe_function_parameters,
    "void f(int a, char *s) { }",
    "f",
    "void(int, *char)"
);
describes!(
    describe_variadic_function,
    "int f(char *s, ...) { return 0; }",
    "f",
    "int(*char, ...)"
);
describes!(describe_pointer_to_function, "int (*p)(void);", "p", "*(int(void))");
describes!(
    describe_array_of_pointer_to_function,
    "int (*p[3])(void);",
    "p",
    "*(int(void))[3]"
);
describes!(describe_array, "int a[3];", "a", "int[3]");
describes!(describe_array_of_array, "int a[3][5];", "a", "int[3][5]");
describes!(describe_array_of_array_of_array, "int a[3][5][7];", "a", "int[3][5][7]");
describes!(describe_incomplete_array, "int a[];", "a", "int[]");
describes!(describe_incomplete_array_of_array, "int a[][5];", "a", "int[][5]");
describes!(describe_array_of_pointer, "int *a[3];", "a", "*int[3]");
describes!(describe_pointer_to_array, "int (*p)[3];", "p", "*(int[3])");
describes!(
    describe_pointer_to_array_of_array,
    "int (*p)[3][5];",
    "p",
    "*(int[3][5])"
);
describes!(describe_array_parameter, "void f(int a[3]) { }", "a", "*int");
describes!(
    describe_function_parameter,
    "void f(int g(void)) { }",
    "g",
    "*(int(void))"
);
describes!(
    describe_adjusted_parameter_types,
    "void f(int a[3], int g(void)) { }",
    "f",
    "void(*int, *(int(void)))"
);
describes!(
    describe_old_style_array_parameter,
    "int f(a) int a[3]; { return 0; }",
    "a",
    "*int"
);
describes!(
    describe_qualified_parameter,
    "void f(const int a) { }",
    "a",
    "const int"
);
describes!(
    describe_unqualified_parameter_type,
    "void f(const int a) { }",
    "f",
    "void(int)"
);
describes!(
    describe_old_style_function,
    "int f(a, b) int a; char b; { return a; }",
    "f",
    "int()"
);

#[test]
fn a_prototype_and_its_definition_declare_one_function() {
    let unit = accepted("int f(int a); int f(int a) { return a; }");
    let functions: Vec<_> = unit
        .symbols()
        .into_iter()
        .filter(|(_, kind, _)| kind == "function")
        .collect();
    assert_eq!(
        functions,
        [("f".to_string(), "function".to_string(), "int(int)".to_string())]
    );
}

#[test]
fn identical_function_types_share_one_interned_type() {
    let unit = accepted("int f(int a); int g(int b); int h(char c);");
    let types: Vec<_> = unit
        .ctx
        .sema
        .symbols
        .data
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function)
        .map(|symbol| symbol.ty.expect("function type").ty)
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
        .data
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
    assert_eq!(unit.ctx.describe(&ty), "int(int, char)");
}

#[test]
fn an_old_style_definition_records_its_parameters_in_declarator_order() {
    let unit = accepted("int f(a, b) char b; { return a; }");
    let def = unit.ctx.sema.functions.data.first().expect("function definition");
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

recover!(
    array_size_is_not_an_integer,
    "int a[1.5];",
    [Diagnosis::NonIntArraySize],
    &[]
);

recover!(
    sizeof_of_an_incomplete_tag,
    "struct S; enum E { A = sizeof(struct S) };",
    [Diagnosis::InvalidSizeof],
    &[]
);

recover!(
    sizeof_of_void,
    "enum E { A = sizeof(void) };",
    [Diagnosis::InvalidSizeof],
    &[]
);

recover!(
    cast_to_a_non_scalar_type,
    "struct S { int a; }; enum E { A = (struct S)1 };",
    [Diagnosis::CastToNonScalar],
    &[]
);
