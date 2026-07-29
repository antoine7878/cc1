%no_main
%start prog
%token<f32> Number
%type<f32> rvalue
%left '\n'
%nonassoc '<'

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
    : rvalue '<' rvalue {if $1 < $3 { 1.0 } else { 0.0 }}
    | Number            {$1}
    ;

%%

pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
