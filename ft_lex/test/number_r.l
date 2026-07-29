%no_main
%no_context
%no_yacc
%tokens Operator(char) Number(i32)

%%

[+\*%]        { return YYToken::Operator(self.yytext.chars().next().unwrap()); }
[0-9]*        { return YYToken::Number(self.yytext.parse::<i32>().unwrap()); }
[ \t\n]*      ;
.             { println!("Unknown token: {}", self.yytext); }

%%

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());
    let mut lexer = YYLex::with_reader(reader);

    loop {
        match lexer.yylex() {
            YYToken::Operator(op) => println!("Operator: {}", op),
            YYToken::Number(n) => println!("Number: {}", n),
            YYToken::yyeof => break,
            _ => (),
        }
    }
}
