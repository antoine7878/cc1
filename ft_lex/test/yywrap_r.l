%no_main

%%

"%%"   print!("COUCOU");

%%

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::stdout;

    let mut i = 0;

    let mut lexer = YYLex::new(File::open("./test/empty.l").unwrap(), stdout(), move || {
        if i < 3 {
            i += 1;
            Some(File::open("./test/empty.l").unwrap())
        } else {
            None
        }
    });

    while let Some(_token) = lexer.next() {}

    Ok(())
}
