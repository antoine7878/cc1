%no_main
%start prog
%type<f32> expr rvalue
%token<f32> Number
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'

%%

prog
    :
    | prog line
    ;

line
    : Number expr '\n' {println!("line {} => {}", $1, $2);}
    ;

expr
    : rvalue {$1}
    ;

rvalue
    : rvalue '+' rvalue
        {
            println!("[line {}] add", $<f32>0);
            $1 + $3
        }
    | rvalue '*' rvalue
        {
            println!("[line {}] mul", $<f32>-2);
            $1 * $3
        }
    | Number {$1}
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
