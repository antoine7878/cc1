%no_main
%start s
%token<f32> Number

%%

s
    : 'a' 'b' 'e' 'n' 't' {println!("charlit ok");}
    ;

%%


fn main() {
    use std::env;
    use std::io::{Cursor, Read};

    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let lexer = lex_yy::YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}

