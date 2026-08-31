%{
/* %} and " inside block comment */
static FRAG_MSG: &str = "esc \" %} ok";
// %} and " inside line comment
static QUOTE: char = '"';
static PCT: char = '%';
%}
%no_main
%start s
%token<f32> Number

%%

s
    : 'x' {println!("{}", FRAG_MSG);}
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
