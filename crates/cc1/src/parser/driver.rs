use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};

use libft::Span;

use crate::context::Context;
use crate::parser::{YYLex, Yacc};
use crate::semantic::{Diagnostic, DiagnosticNode, ExpectedTokens};

pub fn parse_source(ctx: Context) -> Context {
    match File::open(&ctx.file_name) {
        Ok(file) => parse_reader(ctx, BufReader::new(file)).0,
        Err(error) => {
            let mut ctx = ctx;
            ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::InputError(error.to_string()), Span::default()));
            ctx
        }
    }
}

pub fn parse_reader<R: Read>(ctx: Context, reader: R) -> (Context, i32) {
    let lexer = YYLex::new(reader, || None, ctx);
    let mut yacc = Yacc::new(lexer);
    let status = yacc.yyparse();
    (yacc.lexer.ctx, status)
}

pub fn yyerror<D: Display, R: Read>(_msg: D, yacc: &mut Yacc<R>) {
    let span = yacc.lexer.span;
    let found = yacc.yy_lookahead_name();
    let inner = Diagnostic::SyntaxError { found, expected: ExpectedTokens::new(found, &yacc.yy_expected()) };
    yacc.lexer.ctx.diagnostics.push(DiagnosticNode::new(inner, span));
}
