%no_main
%start s
%type<f32> a b c
%token<f32> Number

%%

s
    : a {println!("= {}", $1);}
    ;

a
    : b
    ;

b
    : c
    ;

c
    : Number
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
