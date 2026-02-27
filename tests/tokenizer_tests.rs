use sqlfmt::tokenizer::{tokenize, Token};

#[test]
fn tokenize_select_star() {
    let tokens = tokenize("SELECT * FROM users");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::Keyword("SELECT".into()),
        &Token::Operator("*".into()),
        &Token::Keyword("FROM".into()),
        &Token::Identifier("users".into()),
    ]);
}

#[test]
fn tokenize_string_literal() {
    let tokens = tokenize("'hello world'");
    assert_eq!(tokens, vec![Token::StringLiteral("'hello world'".into())]);
}

#[test]
fn tokenize_number_literal() {
    let tokens = tokenize("42 3.14");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::NumberLiteral("42".into()),
        &Token::NumberLiteral("3.14".into()),
    ]);
}

#[test]
fn tokenize_operators() {
    let tokens = tokenize("a >= b");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::Identifier("a".into()),
        &Token::Operator(">=".into()),
        &Token::Identifier("b".into()),
    ]);
}

#[test]
fn tokenize_parens_comma_semicolon() {
    let tokens = tokenize("(a, b);");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::OpenParen,
        &Token::Identifier("a".into()),
        &Token::Comma,
        &Token::Identifier("b".into()),
        &Token::CloseParen,
        &Token::Semicolon,
    ]);
}

#[test]
fn tokenize_line_comment() {
    let tokens = tokenize("SELECT -- comment\n1");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::Keyword("SELECT".into()),
        &Token::Comment("-- comment".into()),
        &Token::NumberLiteral("1".into()),
    ]);
}

#[test]
fn tokenize_block_comment() {
    let tokens = tokenize("SELECT /* multi\nline */ 1");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::Keyword("SELECT".into()),
        &Token::Comment("/* multi\nline */".into()),
        &Token::NumberLiteral("1".into()),
    ]);
}

#[test]
fn tokenize_quoted_identifier() {
    let tokens = tokenize("\"my table\"");
    assert_eq!(tokens, vec![Token::Identifier("\"my table\"".into())]);
}

#[test]
fn tokenize_case_insensitive_keywords() {
    let tokens = tokenize("select from where");
    let non_ws: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect();
    assert_eq!(non_ws, vec![
        &Token::Keyword("select".into()),
        &Token::Keyword("from".into()),
        &Token::Keyword("where".into()),
    ]);
}
