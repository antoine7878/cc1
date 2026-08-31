%no_main
%start s
%token<f32> Number

%%

s
    : Number {
        /* } in block comment */
        println!("\"{}\"", $1); // } in line comment
        if $1 == 123.0 { println!("brace"); }
    }
    ;

%%

pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
