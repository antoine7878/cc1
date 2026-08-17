use std::io::{self, Cursor, Write};
use std::process::Command;

use cc1::ast::{CompoundStatementNode, ExternalDeclaration, FunctionDefinitionNode, StatementNode};
use cc1::error::CCError;
use cc1::parser::{Context, YYLex, Yacc};

fn parse(src: &str) -> Context {
    let lexer = YYLex::new(Cursor::new(src.as_bytes()), || None, Context::default());
    Yacc::new(lexer).yyparse()
}

fn first_statement(ctx: &Context) -> &StatementNode {
    let decls = &ctx.ast.declarations;
    let ExternalDeclaration::Function(FunctionDefinitionNode {
        body: CompoundStatementNode { statements, .. },
        ..
    }) = &decls[0].decl
    else {
        panic!("expected first declaration to be a function");
    };
    &statements[0]
}

#[test]
fn cc_error_display_and_from() {
    let io_err = io::Error::other("coucou");
    let e: CCError = io_err.into();
    assert_eq!(format!("{e}"), "IO error: coucou");
}

#[test]
fn cc_error_syntax_error_format() {
    let e = CCError::error::<_, ()>("input.c".to_string(), 3, 7, "boom");
    let err = e.expect_err("error() must return Err");
    assert_eq!(format!("{err}"), "Error input.c:3:7: boom");
}

#[test]
fn parsing_invalid_input_recovers_and_calls_yyerror() {
    let ctx = parse("int main(void) { int x = ; return 0; }");
    assert!(
        ctx.ast.declarations.is_empty(),
        "error recovery should not emit declarations"
    );
}

#[test]
fn print_statement_to_stdout() {
    let ctx = parse("int main(void) { return 7; }");
    let stmt = first_statement(&ctx);
    ctx.print_statement(stmt).expect("print_statement");
}

#[test]
fn subprocess_invalid_input_reports_syntax_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cc1"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn cc1");
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"int f( { ").unwrap();
    drop(stdin);
    let res = child.wait_with_output().expect("wait");
    assert!(res.status.success());
    let stderr = String::from_utf8_lossy(&res.stderr);
    assert!(stderr.contains("syntax error"), "stderr was: {stderr}");
}

#[test]
fn subprocess_valid_input_prints_ast() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cc1"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn cc1");
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"int f(void) { return 0; }").unwrap();
    drop(stdin);
    let res = child.wait_with_output().expect("wait");
    assert!(res.status.success());
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(
        stdout.contains("TranslationUnitDecl") && stdout.contains("FunctionDecl"),
        "stdout was: {stdout}"
    );
}
