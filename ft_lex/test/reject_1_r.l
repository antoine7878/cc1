%no_main
%no_context
%no_yacc

%%

salut   print!("1"); self.reject();
salu    print!("2"); self.reject();
sal     print!("3"); self.reject();
sa      print!("4"); self.reject();
s       print!("5"); self.reject();

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let mut lexer = YYLex::with_reader(reader);

    while lexer.yylex() != YYToken::yyeof {}
}
