%no_main
%start prog
%type<f32> rvalue
%token<f32> Number
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '('

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
    : rvalue '+' rvalue {$1 + $3}
    | rvalue '-' rvalue {$1 - $3}
    | rvalue '*' rvalue {$1 * $3}
    | rvalue '/' rvalue {$1 / $3}
    | '(' rvalue ')'         {$2}
    | Number                 {$1}
    ;

%%

pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
