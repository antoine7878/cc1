%no_main

%%

"/*"  {
    let mut prev: u8 = 0;
    loop {
        match (self.input(), prev) {
            (0, _) => break,
            (b'/', b'*') => break,
            (c, _) => prev = c,
        }
    }
}

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let mut lexer = YYLex::with_reader(reader);

    while lexer.yylex() != YYToken::yyeof {}
}
