%no_main
%start prog
%token<f32> Number
w
%left '\n'

%%

prog
    : item
    | item prog
    ;

item
    : 'o' '\n'   {println!("ok");}
    | 'q'        {self.yyclearin(); self.yyaccept();}
    | 'x'        {self.yyabort();}
    | error '\n' {self.yyerrok(); println!("recovered {}", self.yyrecovering());}
    ;

%%

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let reader = Cursor::new(args[1].clone());

    let lexer = lex_yy::YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    let ret = yacc.yyparse();
    println!("ret={}", ret);
}
