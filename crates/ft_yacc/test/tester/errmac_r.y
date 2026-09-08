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
    let ret = yacc.yyparse();
    println!("ret={}", ret);
}
