%no_main
%start prog
%token<f32> Number
%type<f32> rvalue
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'


%%

prog
    : expr
    | expr '\n' prog
    ;

expr
    : rvalue {println!("= {}", $1);}
    ;

rvalue
    : rvalue '+' rvalue {$1 + $3}  %prec '*'
    | rvalue '-' rvalue {$1 - $3}
    | rvalue '*' rvalue {$1 * $3}
    | rvalue '/' rvalue {$1 / $3}
    | '(' rvalue ')'         {$2}
    | Number                 {$1}
    ;

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};

    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
