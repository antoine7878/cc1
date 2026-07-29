%no_main
%no_context
%no_yacc

%%

"%%"   print!("COUCOU");

%%

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::stdout;

    let mut i = 0;

    let mut lexer = YYLex::new(File::open("./test/empty.l").unwrap(), move || {
        if i < 3 {
            i += 1;
            Some(File::open("./test/empty.l").unwrap())
        } else {
            None
        }
    }, Context{});

    while lexer.yylex() != YYToken::yyeof {}

    Ok(())
}
