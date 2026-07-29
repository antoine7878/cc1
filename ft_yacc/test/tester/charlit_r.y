%no_main
%start s
%token<f32> Number

%%

s
    : 'a' 'b' 'e' 'n' 't' {println!("charlit ok");}
    ;

%%

pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::env;
    use std::io::{Cursor, Read};

    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}

