%no_main
%start prog
%token<i32> Int
%token<f64> Real
%type<f64> expr
%left '\n'
%left '+'

%%

prog
    : expr {println!("= {}", $1);}
    | expr '\n' prog {println!("= {}", $1);}
    |
    ;

expr
    : expr '+' expr {$1 + $3}
    | Real          {$1}
    | Int           {$1 as f64}
    ;

%%

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let reader = Cursor::new(args[1].clone());

    let lexer = lex_yy::YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
