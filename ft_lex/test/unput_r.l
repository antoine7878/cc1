%no_main

%%

salut  print!("MATCH"); let _ = self.input(); let _ = self.input(); self.unput(b'1'); self.unput(b'2');

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let mut lexer = YYLex::with_reader(reader);

    while lexer.yylex() != YYToken::yyeof {}
}
