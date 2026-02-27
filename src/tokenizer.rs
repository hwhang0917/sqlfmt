#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    Operator(String),
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
    Comment(String),
    Whitespace(String),
    Other(String),
}

pub fn tokenize(_input: &str) -> Vec<Token> {
    vec![]
}
