%no_main

%%

salut       print!("{}", self.yytext); self.yymore();
bonjour     print!("{}", self.yytext);

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let mut lexer = YYLex::with_reader(reader);

    while lexer.yylex() != YYToken::yyeof {}
}
