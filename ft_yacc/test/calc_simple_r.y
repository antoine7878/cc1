%no_main
%start expr
%token Number(f32) lvalue(f32)
%left '+'
%left '*'


%%

expr
    : lvalue {println!("= {}", $1);}
    ;

lvalue
    : lvalue '+' lvalue {$1 + $3}
    | lvalue '*' lvalue {$1 * $3}
    | Number                 {$1}
    ;

%%

fn main() {
    let input = [
        YYToken::Number(2.),
        YYToken::Char('+'),
        YYToken::Number(3.),
        YYToken::Char('*'),
        YYToken::Number(4.),
        YYToken::yyeof,
    ];
    let mut yacc = Yacc::new(input);
    match yacc.yyparse() {
        Ok(()) => (),
        Err(e) => println!("error: {e}"),
    }
}
