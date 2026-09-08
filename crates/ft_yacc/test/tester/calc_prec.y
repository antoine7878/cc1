%no_main
%start prog
%token<i32> NUMBER
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'
%type<i32> expr rvalue


%%

prog
    : expr
    | expr '\n' prog
    | error '\n' prog
    ;

expr
    : rvalue {println!("= {}", $1);}
    ;

rvalue
    : rvalue '+' rvalue {$1 + $3} %prec '*'
    | rvalue '-' rvalue {$1 - $3}
    | rvalue '*' rvalue {$1 * $3}
    | rvalue '/' rvalue {$1 / $3}
    | '(' rvalue ')' {$2}
    | NUMBER
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
