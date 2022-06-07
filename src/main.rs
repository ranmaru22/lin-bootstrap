use std::fs;

mod lexer;

fn run(text: &str) -> Result<Vec<lexer::Token>, lexer::LexerError> {
    let mut lexer = lexer::Lexer::new(text);

    lexer.tokenize()
}

fn main() -> std::io::Result<()> {
    let file = fs::read_to_string("examples/01.lin")?;

    match run(&file) {
        Ok(tokens) => {
            println!("{:?}", tokens);
            Ok(())
        }
        Err(err) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        )),
    }
}
