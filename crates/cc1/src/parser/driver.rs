use std::fmt::Display;
use std::fs::read;
use std::io::{Cursor, Read};

use libft::Span;

use crate::context::Context;
use crate::parser::depth::{MAX_EXPRESSION_DEPTH, deepest_expression};
use crate::parser::{YYLex, Yacc};
use crate::semantic::{Diagnostic, DiagnosticNode, ExpectedTokens};

pub fn parse_source(ctx: Context) -> Context {
    match read(&ctx.file_name) {
        Ok(bytes) => {
            let bytes: Vec<u8> = bytes.into_iter().filter(|&b| b != 0).collect();
            let text = decode_source(&bytes);
            parse_reader(ctx, Cursor::new(text)).0
        }
        Err(error) => {
            let mut ctx = ctx;
            ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::InputError(error.to_string()), Span::default()));
            ctx
        }
    }
}

fn decode_source(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len());
    let mut rest = bytes;
    while !rest.is_empty() {
        match std::str::from_utf8(rest) {
            Ok(valid) => {
                text.push_str(valid);
                break;
            }
            Err(error) => {
                let (valid, tail) = rest.split_at(error.valid_up_to());
                text.push_str(std::str::from_utf8(valid).unwrap());
                let bad = error.error_len().unwrap_or(tail.len());
                text.extend(tail[..bad].iter().map(|&b| crate::ast::escape::raw_char(b)));
                rest = &tail[bad..];
            }
        }
    }
    text
}

pub fn parse_reader<R: Read>(ctx: Context, reader: R) -> (Context, i32) {
    let lexer = YYLex::new(reader, || None, ctx);
    let mut yacc = Yacc::new(lexer);
    let status = yacc.yyparse();
    let mut ctx = yacc.lexer.ctx;
    if let Some(span) = deepest_expression(&ctx.arenas.expressions, MAX_EXPRESSION_DEPTH + 2) {
        ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::ExpressionTooDeep(MAX_EXPRESSION_DEPTH), span));
    }
    (ctx, status)
}

pub fn yyerror<D: Display, R: Read>(_msg: D, yacc: &mut Yacc<R>) {
    let span = yacc.lexer.span;
    let found = yacc.yy_lookahead_name();
    let inner = Diagnostic::SyntaxError { found, expected: ExpectedTokens::new(found, &yacc.yy_expected()) };
    yacc.lexer.ctx.diagnostics.push(DiagnosticNode::new(inner, span));
}
