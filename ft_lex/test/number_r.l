%no_main
%tokens Operator(char) Number(i32) Other(String)

%%

[+\*%]        return YYToken::Operator(self.yytext.chars().next().unwrap());
[0-9]*        return YYToken::Number(self.yytext.parse::<i32>().unwrap());
[ \t\n]*      {}
.             println!("Unknown token: {}", self.yytext);

%%

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut lexer = YYLex::default();

    while let Some(token) = lexer.next() {
        match token {
            YYToken::Operator(op) => println!("Operator: {}", op),
            YYToken::Number(n) => println!("Number: {}", n),
            YYToken::Other(s) => println!("OTHER: {}", s),
            _ => (),
        }
    }

    Ok(())
}

