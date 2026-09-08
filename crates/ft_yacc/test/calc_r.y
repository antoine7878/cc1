%no_main

%{ mod lex_yy;
mod context;

use std::io::stdin;
use context::Context;
%}

%start prog
%type<f32> rvalue %token<f32> Number
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'

%%

prog
    : expr
    | expr '\n' prog
    | error '\n' prog
    |
    ;

expr
    : rvalue { println!("= {}", $1); }
    ;

rvalue
    : rvalue '+' rvalue { self.lexer.ctx().increment_op(); $1 + $3 }
    | rvalue '-' rvalue { self.lexer.ctx().increment_op(); $1 - $3 }
    | rvalue '*' rvalue { self.lexer.ctx().increment_op(); $1 * $3 }
    | rvalue '/' rvalue
    {
        self.lexer.ctx().increment_op();
        if $3 == 0.0 {
            return YYToken::error;
        }
        $1 / $3
    }
    | '(' rvalue ')'         { $2 }
    | Number                 { self.lexer.ctx().increment_num(); $1 }
    ;

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());
    let mut lexer = lex_yy::YYLex::new(reader, || None, Context::new());

    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
    println!("ctx: {:?}", yacc.lexer.ctx);
}
