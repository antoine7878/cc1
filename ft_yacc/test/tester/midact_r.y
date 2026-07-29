%no_main
%start s
%token<f32> Number

%%

s
    : 'x' Number {println!("mid {}", $2);} 'y' {println!("end {}", $2);}
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
