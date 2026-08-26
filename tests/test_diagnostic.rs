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
    report_every_diagnosis_in_order,
    "int f(a, b) int a; int a; { return a; }",
    [
        "<test>:1:24: error: duplicate declaration of parameter `a'",
        "<test>:1:5: error: Absctract declaration in old style function",
    ]
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
    assert!(rendered.contains("error: "), "{rendered:?}");
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
    report_empty_struct_declaration,
    "struct s { int a; int; };",
    ["<test>:1:19: error: Declaration declares nothing"]
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
