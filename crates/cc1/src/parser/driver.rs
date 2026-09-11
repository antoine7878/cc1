use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};

use crate::context::Context;
use crate::parser::{YYLex, Yacc};
use crate::semantic::{Diagnosis, DiagnosisNode, ExpectedTokens};

pub fn parse_source(ctx: Context) -> Context {
    let file = File::open(&ctx.file_name).unwrap();
    parse_reader(ctx, BufReader::new(file)).0
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
    let inner = Diagnosis::SyntaxError { found, expected: ExpectedTokens::new(found, &yacc.yy_expected()) };
    yacc.lexer.ctx.diagnosis.push(DiagnosisNode::new(inner, span));
}
