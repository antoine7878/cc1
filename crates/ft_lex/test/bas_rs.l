%no_main
%tokens Operator Number

%%

[+\*%]          return YYToken::Operator;
[0-9]+          return YYToken::Number;
[ \t\n]+        return YYToken::Default;
.               return YYToken::Default;


%%

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env::args;
    use std::fs::File;
    use std::io::stdout;


    let file: File = File::open(args().nth(1).unwrap())?;

    let mut i = 1;

    let mut lexer = YYLex::new(file, stdout(), move |lex| {
        println!("i: {}", i);
        if i < 4 {
            i += 1;
            lex.yyin = File::open(args().nth(1).unwrap()).unwrap();
            false
        } else {
            true
        }
    });

    while let Some(token) = lexer.next() {
        match token {
            Ok(YYToken::Operator) => println!("Operator: {}", lexer.yytext),
            Ok(YYToken::Number) => println!("Number: {}", lexer.yytext),
            Ok(YYToken::Default) => println!("OTHER"),
            Ok(YYToken::Reject) => unreachable!(),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
