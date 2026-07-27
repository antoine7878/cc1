%no_main
%start prog
%token<f32> Number
%type<f32> rvalue
%left '\n'
%left '+' '-'
%right '^'

%%

prog
    : expr
    | expr '\n' prog
    ;

expr
    : rvalue {println!("= {}", $1);}
    ;

rvalue
    : rvalue '+' rvalue {$1 + $3}
    | rvalue '-' rvalue {$1 - $3}
    | rvalue '^' rvalue {$1.powf($3)}
    | '(' rvalue ')'    {$2}
    | Number            {$1}
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
