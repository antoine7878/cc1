%no_main
%no_context
%no_yacc

%%

a*   print!("1"); self.reject();
a*   print!("2"); self.reject();
a*   print!("3");

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let mut lexer = YYLex::with_reader(reader);

    while lexer.yylex() != YYToken::yyeof {}
}
