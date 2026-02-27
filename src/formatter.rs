use crate::tokenizer::Token;

pub fn beautify(_tokens: &[Token]) -> String {
    String::new()
}

#[derive(Clone, Copy, PartialEq)]
enum PrevToken {
    None,
    Keyword,
    Word,
    Operator,
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
}

fn needs_space(prev: PrevToken, token: &Token) -> bool {
    match token {
        Token::Keyword(_) => matches!(
            prev,
            PrevToken::Keyword | PrevToken::Word | PrevToken::Operator | PrevToken::CloseParen
        ),
        Token::Identifier(_) | Token::StringLiteral(_) | Token::NumberLiteral(_) | Token::Other(_) => {
            matches!(
                prev,
                PrevToken::Keyword | PrevToken::Word | PrevToken::CloseParen
            )
        }
        Token::Operator(_) => prev == PrevToken::Keyword,
        _ => false,
    }
}

pub fn minify(tokens: &[Token]) -> String {
    let mut out = String::new();
    let mut prev = PrevToken::None;

    for token in tokens {
        match token {
            Token::Whitespace(_) | Token::Comment(_) => continue,
            _ => {}
        }

        if needs_space(prev, token) {
            out.push(' ');
        }

        match token {
            Token::Keyword(kw) => {
                out.push_str(&kw.to_uppercase());
                prev = PrevToken::Keyword;
            }
            Token::Identifier(id) => {
                out.push_str(id);
                prev = PrevToken::Word;
            }
            Token::StringLiteral(s) => {
                out.push_str(s);
                prev = PrevToken::Word;
            }
            Token::NumberLiteral(n) => {
                out.push_str(n);
                prev = PrevToken::Word;
            }
            Token::Operator(op) => {
                out.push_str(op);
                prev = PrevToken::Operator;
            }
            Token::Comma => {
                out.push(',');
                prev = PrevToken::Comma;
            }
            Token::Semicolon => {
                out.push(';');
                prev = PrevToken::Semicolon;
            }
            Token::OpenParen => {
                out.push('(');
                prev = PrevToken::OpenParen;
            }
            Token::CloseParen => {
                out.push(')');
                prev = PrevToken::CloseParen;
            }
            Token::Other(o) => {
                out.push_str(o);
                prev = PrevToken::Word;
            }
            Token::Whitespace(_) | Token::Comment(_) => unreachable!(),
        }
    }

    out.trim_end().to_string()
}
