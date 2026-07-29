%no_main
%start second
%token<f32> Number
%type<f32> first

%%

first
    : 'x' Number {$2 + 100.0}
    ;

second
    : Number {println!("= {}", $1);}
    ;

%%

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
