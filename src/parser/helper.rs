use std::fs::File;
use std::{env::args, io::BufReader};

use crate::parser::{Context, Span, YYLex, Yacc};
use crate::semantic::{Diagnosis, DiagnosisNode};

pub fn parse_args(mut ctx: Context) -> Context {
    println!();
    let mut args = args();
    if args.next().is_none() {
        ctx.diagnosis
            .push(DiagnosisNode::new(Diagnosis::BadArgumentsCount, Span::default()));
        return ctx;
    };
    let Some(file_name) = args.next() else {
        ctx.diagnosis
            .push(DiagnosisNode::new(Diagnosis::BadArgumentsCount, Span::default()));
        return ctx;
    };
    ctx.set_file_name(file_name);
    ctx
}

pub fn parse_source(ctx: Context) -> Context {
    let file = File::open(&ctx.file_name).unwrap();
    let lexer = YYLex::new(BufReader::new(file), || None, ctx);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
    yacc.lexer.ctx
}
