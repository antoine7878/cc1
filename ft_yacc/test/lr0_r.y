%no_main
%start s
%token e(u32) id(u32) t(u32) '+' '(' ')'

%%

s
    : e {println!("res: {}", $1);}
    ;

e
    : e '+' t {$1 + $3}
    | t {$1}
    ;

t
    : '(' e ')' {$2}
    | id {$1}
    ;

%%

fn main() {
    let mut yacc = Yacc::new();
    let input = [
        YYToken::Char('('),
        YYToken::id(1),
        YYToken::Char('+'),
        YYToken::id(2),
        YYToken::Char(')'),
        YYToken::Char('+'),
        YYToken::id(3),
        YYToken::yyeof,
    ];
    match yacc.yyparse(input.iter().cloned()) {
        Ok(()) => (),
        Err(e) => println!("error: {e}"),
    }
}
