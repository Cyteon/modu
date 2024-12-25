use logos::Logos;

#[derive(Default, Debug, PartialEq, Clone)]
pub enum LexingError {
    #[default]
    UnexpectedToken,
    InvalidInteger(String),
    ExpectedToken,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(err: std::num::ParseIntError) -> Self {
        use std::num::IntErrorKind::*;

        match err.kind() {
            PosOverflow | NegOverflow => LexingError::InvalidInteger("Integer overflow".to_string()),
            _ => LexingError::InvalidInteger("Other error".to_string()),
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error = LexingError)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex("//[^\n]*|/\\*([^*]|\\*[^/])*\\*/")]
    Comment,

    #[token("let")]
    Let,

    #[token("fn")]
    Fn,

    #[token("import")]
    Import,

    #[token("return")]
    Return,

    #[token("as")]
    As,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("if")]
    If,

    #[token(".")]
    Dot,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifer,

    #[token("=")]
    Assign,

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(i64),

    #[regex("[0-9]+\\.[0-9]+")]
    Float,

    #[regex(r#""[^"]*""#)]
    String,

    #[regex("true|false")]
    Boolean,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBracket,

    #[token("}")]
    RBracket,

    #[token("==")]
    IsEqual,

    #[token("!=")]
    IsUnequal,

    #[token("*")]
    Star,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asing_str() {
        let mut lexer = Token::lexer("let x = \"test\"");
        assert_eq!(lexer.next(), Some(Ok(Token::Let)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifer)));
        assert_eq!(lexer.next(), Some(Ok(Token::Assign)));
        assert_eq!(lexer.next(), Some(Ok(Token::String)));
    }

    #[test]
    fn asing_number() {
        let mut lexer = Token::lexer("let x = 10");
        assert_eq!(lexer.next(), Some(Ok(Token::Let)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifer)));
        assert_eq!(lexer.next(), Some(Ok(Token::Assign)));
        assert_eq!(lexer.next(), Some(Ok(Token::Number(10))));
    }

    #[test]
    fn asing_boolean() {
        let mut lexer = Token::lexer("let x = true");
        assert_eq!(lexer.next(), Some(Ok(Token::Let)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifer)));
        assert_eq!(lexer.next(), Some(Ok(Token::Assign)));
        assert_eq!(lexer.next(), Some(Ok(Token::Boolean)));
    }

    #[test]
    fn expr() {
        let lexer = Token::lexer("print(\"Hello, world!\")");

        for token in lexer {
            println!("{:?}", token);
        }
    }
}
