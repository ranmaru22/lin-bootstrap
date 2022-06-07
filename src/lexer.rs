use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

static LEGAL_EXIT_CHARS: [char; 1] = ['}'];
static RESERVED_CHARS: [char; 2] = ['\x27', '{'];

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    EOF,
    Function(String),
    Int(i64),
    Float(f64),
    Symbol(String),
    String(String),
    OpeningBrace,
    ClosingBrace,
}

pub enum LexerError {
    InvalidNumber,
    InvalidToken,
    InvalidSymbolName,
    UnterminatedString,
}

impl fmt::Debug for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidToken => write!(f, "invalid token"),
            Self::InvalidSymbolName => write!(f, "invalid symbol name"),
            Self::UnterminatedString => write!(f, "unterminated string"),
        }
    }
}

impl From<std::num::ParseIntError> for LexerError {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::InvalidNumber
    }
}

impl From<std::num::ParseFloatError> for LexerError {
    fn from(_: std::num::ParseFloatError) -> Self {
        Self::InvalidNumber
    }
}

impl std::error::Error for LexerError {}

enum Either<L, R> {
    Left(L),
    Right(R),
}

pub struct Lexer<'s> {
    text: Peekable<Chars<'s>>,
}

impl<'s> Lexer<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            text: text.chars().peekable(),
        }
    }

    fn make_number(&mut self, ch: char) -> Result<Either<i64, f64>, LexerError> {
        let mut has_dot = ch == '.';
        let mut num_str = String::from(ch);

        while let Some(ch) = self.text.peek() {
            if LEGAL_EXIT_CHARS.contains(ch) {
                break;
            } else if RESERVED_CHARS.contains(ch) {
                return Err(LexerError::InvalidNumber)
            }

            match self.text.next() {
                None => break,
                Some(ch) if ch.is_ascii_whitespace() => break,

                Some('.') => {
                    if has_dot {
                        return Err(LexerError::InvalidNumber);
                    }

                    has_dot = true;
                    num_str.push('.');
                }

                Some(ch) => num_str.push(ch),
            }
        }

        if has_dot {
            Ok(Either::Right(num_str.parse::<f64>()?))
        } else {
            Ok(Either::Left(num_str.parse::<i64>()?))
        }
    }

    fn make_string(&mut self) -> Result<String, LexerError> {
        let mut str = String::new();
        let mut terminated = false;

        while let Some(ch) = self.text.next() {
            if ch == '"' {
                terminated = true;
                break;
            } else {
                str.push(ch);
            }
        }

        if terminated {
            Ok(str)
        } else {
            Err(LexerError::UnterminatedString)
        }
    }

    fn make_symbol(&mut self, entry_char: Option<char>) -> Result<String, LexerError> {
        let mut symbol_name = match entry_char {
            Some(ch) => String::from(ch),
            None => String::new(),
        };

        while let Some(ch) = self.text.peek() {
            if LEGAL_EXIT_CHARS.contains(ch) || ch.is_ascii_whitespace() {
                break;
            } else if RESERVED_CHARS.contains(ch) {
                return Err(LexerError::InvalidSymbolName);
            }

            // SAFETY: We just peeked. This is safe.
            let ch = self.text.next().unwrap();
            symbol_name.push(ch);
        }

        Ok(symbol_name)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.text.next() {
            match ch {
                // Skip all whitespace
                ch if ch.is_ascii_whitespace() => (),

                // Numbers
                ch if ch.is_digit(10) || ch == '.' => match self.make_number(ch) {
                    Ok(Either::Left(int)) => tokens.push(Token::Int(int)),
                    Ok(Either::Right(float)) => tokens.push(Token::Float(float)),
                    Err(err) => return Err(err),
                },

                // Symbols
                '\x27' => match self.make_symbol(None) {
                    Ok(symbol) => tokens.push(Token::Symbol(symbol)),
                    Err(err) => return Err(err),
                },

                // Strings
                '"' => match self.make_string() {
                    Ok(string) => tokens.push(Token::String(string)),
                    Err(err) => return Err(err),
                },

                // Quotations
                '{' => tokens.push(Token::OpeningBrace),
                '}' => tokens.push(Token::ClosingBrace),

                // Function calls
                ch if ch.is_ascii() => match self.make_symbol(Some(ch)) {
                    Ok(symbol) => tokens.push(Token::Function(symbol)),
                    Err(err) => return Err(err),
                }

                _ => return Err(LexerError::InvalidToken),
            }
        }

        tokens.push(Token::EOF);
        Ok(tokens)
    }
}
