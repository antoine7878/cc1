%no_main
%union {
    coucou salut;
    bonjour  hello;
}
%start prog
%type<f32> rvalue
%token<f32> Number
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'

%{
mod lex_yy;
%}

%%

prog
    : expr
    | expr '\n' prog
    | error '\n' prog
    |
    ;

expr
    : rvalue {println!("= {}", $1);}
    ;

rvalue
    : rvalue '+' rvalue {$1 + $3}
    | rvalue '-' rvalue {$1 - $$}
    | rvalue '*' rvalue {$1 * $3}
    | rvalue '/' rvalue
    {
        if $3 == 0.0 {
            return YYToken::error;
        }
        $1 / $3
    }
    | '(' rvalue ')'         {$2}
    | Number                 {$1}
    ;

%%

fn main() {
    use std::process::exit;

    let lexer = lex_yy::YYLex::default();
    let mut yacc = Yacc::new(lexer);
    yacc.yydebug = true;
    exit(yacc.yyparse());
}
