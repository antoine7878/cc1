use std::io::Cursor;

use cc1::context::Context;
use cc1::parser::{YYLex, Yacc};

/// Parse a C source string and return the printed AST dump.
fn dump(src: &str) -> String {
    let lexer = YYLex::new(Cursor::new(src.as_bytes()), || None, Context::default());
    let ctx = Yacc::new(lexer).yyparse();
    let mut out = Vec::new();
    ctx.print_ast(&mut out).expect("print_ast");
    String::from_utf8(out).expect("print_ast produced non-UTF8 output")
}

/// Assert that parsing `src` produces the expected AST dump.
fn check(src: &str, expected: &str) {
    let actual = dump(src);
    if actual == expected {
        return;
    }
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    let first_diff = expected_lines
        .iter()
        .zip(actual_lines.iter())
        .position(|(e, a)| e != a);
    match first_diff {
        Some(i) => panic!(
            "AST mismatch at line {}\n  expected: {:?}\n  actual:   {:?}\n\nsource:\n{}\n---\nexpected:\n{}\nactual:\n{}",
            i + 1,
            expected_lines.get(i),
            actual_lines.get(i),
            src,
            expected,
            actual,
        ),
        None => panic!(
            "AST line count differs (expected {}, actual {})\n\nsource:\n{}\n---\nexpected:\n{}\nactual:\n{}",
            expected_lines.len(),
            actual_lines.len(),
            src,
            expected,
            actual,
        ),
    }
}

/// Generate one test per golden-file pair.
macro_rules! ast_test {
    ($name:ident, $src:literal, $ast:literal) => {
        #[test]
        fn $name() {
            check(include_str!($src), include_str!($ast));
        }
    };
}

ast_test!(primitives, "../test/ast/primitives.c", "../test/ast/primitives.ast");
ast_test!(pointers, "../test/ast/pointers.c", "../test/ast/pointers.ast");
ast_test!(storage, "../test/ast/storage.c", "../test/ast/storage.ast");
ast_test!(typedefs, "../test/ast/typedefs.c", "../test/ast/typedefs.ast");
ast_test!(structs, "../test/ast/structs.c", "../test/ast/structs.ast");
ast_test!(enums, "../test/ast/enums.c", "../test/ast/enums.ast");
ast_test!(bitfields, "../test/ast/bitfields.c", "../test/ast/bitfields.ast");
ast_test!(initializers, "../test/ast/initializers.c", "../test/ast/initializers.ast");
ast_test!(functions, "../test/ast/functions.c", "../test/ast/functions.ast");
ast_test!(strings, "../test/ast/strings.c", "../test/ast/strings.ast");
ast_test!(exprs, "../test/ast/exprs.c", "../test/ast/exprs.ast");
ast_test!(stmts, "../test/ast/stmts.c", "../test/ast/stmts.ast");
