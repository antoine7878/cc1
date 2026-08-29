mod common;

use common::{Unit, strip_ansi};

fn messages(src: &str) -> Vec<String> {
    Unit::compile(src).messages()
}

macro_rules! reports {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(messages($src), $expected, "{:?}", $src);
        }
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            assert_eq!(messages($src), $expected, "{:?}", $src);
        }
    };
}

reports!(
    report_undeclared_identifier,
    "void f(void) { x = 1; }",
    ["<test>:1:16: error: Use of undeclared identifier 'x'"]
);

reports!(
    report_points_at_the_offending_line,
    "int f(void)\n{\n  return y;\n}",
    ["<test>:3:10: error: Use of undeclared identifier 'y'"]
);

reports!(
    report_duplicate_declaration,
    "int x; char x;",
    ["<test>:1:13: error: duplicate declaration of variable `x'"]
);

reports!(
    report_duplicate_declaration_across_lines,
    "int x;\nchar x;",
    ["<test>:2:6: error: duplicate declaration of variable `x'"]
);

reports!(
    report_non_constant_expression,
    "int x; enum E { A = x };",
    ["<test>:1:21: error: Non constant expression"]
);

reports!(
    report_non_integer_constant_expression,
    "enum E { A = 1.5 };",
    ["<test>:1:14: error: Non integer constant expression"]
);

reports!(
    report_non_integral_bit_field,
    "struct S { double a : 3; };",
    ["<test>:1:1: error: Bit-field has non-integral type"]
);

reports!(
    report_old_style_parameter_declared_twice,
    "int f(a, b) int a; int a; { return a; }",
    ["<test>:1:24: error: duplicate declaration of parameter `a'"]
);

reports!(
    report_void_is_not_the_only_parameter,
    "void f(void, int) { }",
    [
        "<test>:1:8: error: Parameter shall not have void type",
        "<test>:1:6: error: Parameter shall include an identifier",
    ]
);

reports!(
    report_named_void_parameter,
    "void f(void x) { }",
    ["<test>:1:8: error: Parameter shall not have void type"]
);

reports!(
    report_parameter_storage_class,
    "void f(static int a) { }",
    ["<test>:1:8: error: Parameter shall only by declared with register storage"]
);

reports!(
    report_prototype_with_declaration_list,
    "int f(int a) int b; { return a; }",
    ["<test>:1:5: error: Parameter style function declration shall not be followed by a declaration list"]
);

reports!(
    report_duplicate_parameter,
    "int f(int a, int a) { return a; }",
    ["<test>:1:18: error: duplicate declaration of parameter `a'"]
);

reports!(
    report_declarator_is_not_a_function,
    "int (*f)(void) { return 0; }",
    ["<test>:1:5: error: Declarator shall be function type"]
);

#[test]
fn an_accepted_unit_reports_nothing() {
    let unit = Unit::compile("int main(void) { return 0; }");
    assert!(unit.messages().is_empty());
    assert_eq!(unit.render(), "");
}

#[test]
fn a_syntax_error_is_reported_by_the_parser() {
    let unit = Unit::compile("int f(void) { return; ; }; }");
    assert!(!unit.parsed());
    assert!(
        unit.messages().iter().any(|message| message.contains("syntax error")),
        "{:?}",
        unit.messages()
    );
}

#[test]
fn a_report_is_colored_by_severity() {
    let unit = Unit::compile("void f(void) { x = 1; }");
    let rendered = unit.render();
    assert!(rendered.contains("\x1b[0;31m"), "{rendered:?}");
    assert!(strip_ansi(&rendered).contains("error: "), "{rendered:?}");
    assert_eq!(strip_ansi(&rendered).trim_end(), unit.messages()[0]);
}

#[test]
fn a_report_names_the_file_and_position() {
    let unit = Unit::compile("void f(void) { x = 1; }");
    let message = &unit.messages()[0];
    assert!(message.starts_with("<test>:1:16: "), "{message}");
}

reports!(
    report_empty_declaration,
    "int ;",
    ["<test>:1:1: error: Declaration declares nothing"]
);

reports!(
    report_syntax_error_lists_expected_tokens,
    "struct s { int a }",
    ["<test>:1:18: error: syntax error, unexpected '}', expecting ',' or ';'"]
);

reports!(
    report_struct_error_recovers_at_the_next_member,
    "struct s { int a; : 0; int b; };",
    ["<test>:1:19: error: syntax error, unexpected ':'"]
);

reports!(
    report_struct_error_recovers_on_a_leading_member,
    "struct s { : 0; int a; };",
    ["<test>:1:12: error: syntax error, unexpected ':'"]
);

reports!(
    report_syntax_error_at_end_of_file,
    "int f(void) { return 0; } }",
    ["<test>:1:27: error: syntax error, unexpected '}', expecting end of file"]
);

reports!(
    report_unterminated_block_expects_a_closing_brace,
    "int f(void) { return 0;",
    ["<test>:1:24: error: syntax error, unexpected end of file, expecting ';' or '}'"]
);

reports!(
    report_syntax_error_without_expected_tokens,
    "int x = ;",
    ["<test>:1:9: error: syntax error, unexpected ';'"]
);

reports!(
    report_struct_without_member,
    "struct s { int; };",
    [
        "<test>:1:12: error: Declaration declares nothing",
        "<test>:1:1: error: struct has no named member"
    ]
);

reports!(
    report_union_without_member,
    "union u { int; };",
    [
        "<test>:1:11: error: Declaration declares nothing",
        "<test>:1:1: error: union has no named member"
    ]
);

reports!(
    report_struct_with_only_unnamed_bit_fields,
    "struct t { int : 3; };",
    ["<test>:1:1: error: struct has no named member"]
);

reports!(
    report_empty_struct_declaration,
    "struct s { int a; int; };",
    ["<test>:1:19: error: Declaration declares nothing"]
);

reports!(
    report_integer_constant_too_large,
    "int x = 0x100000000;",
    ["<test>:1:9: warning: integer constant is too large for any integer type"]
);

reports!(
    report_concat_narrow_then_wide_literal,
    "int f(void) { return (\"a\" L\"b\")[0]; }",
    ["<test>:1:23: warning: concatenation of a wide and a narrow string literal is undefined"]
);

reports!(
    report_concat_wide_then_narrow_literal,
    "int f(void) { return (L\"a\" \"b\")[0]; }",
    ["<test>:1:23: warning: concatenation of a wide and a narrow string literal is undefined"]
);

reports!(
    report_concat_wide_literal_in_subscript,
    "int f(void) { return L\"ab\" \"cd\"[0]; }",
    ["<test>:1:22: warning: concatenation of a wide and a narrow string literal is undefined"]
);

reports!(
    report_arithmetic_overflow_in_addition,
    "enum E { A = 2147483647 + 1 };",
    ["<test>:1:14: warning: integer overflow in constant expression"]
);

reports!(
    report_arithmetic_overflow_in_negation,
    "enum E { A = -(-2147483647 - 1) };",
    ["<test>:1:14: warning: integer overflow in constant expression"]
);

reports!(
    report_remainder_by_zero,
    "enum E { A = 1 % 0 };",
    ["<test>:1:14: error: Division by zero"]
);

reports!(
    report_constant_overflow,
    "enum E { A = (-2147483647 - 1) / -1 };",
    ["<test>:1:14: error: overflow in constant expression"]
);

reports!(
    report_external_register,
    "register int x;",
    ["<test>:1:1: error: External declaration auto of register"]
);

reports!(
    report_block_function_not_extern,
    "void f(void) { auto int g(void); }",
    ["<test>:1:25: error: Function in block not declared as extern"]
);
