fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = YYLex::default();

    while let Some(_token) = lexer.next() {}
    Ok(())
}
