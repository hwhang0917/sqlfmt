use sqlfmt::formatter::{beautify, colorize, Palette};
use sqlfmt::tokenizer::tokenize;

#[test]
fn colorize_none_is_identity() {
    let tokens = tokenize("SELECT id FROM users WHERE id = 1;");
    let formatted = beautify(&tokens);
    assert_eq!(colorize(&formatted, &Palette::none()), formatted);
}

#[test]
fn colorize_wraps_keywords() {
    let tokens = tokenize("SELECT 1;");
    let formatted = beautify(&tokens);
    let colored = colorize(&formatted, &Palette::ansi());
    assert!(colored.contains("\x1b[1;36mSELECT\x1b[0m"));
}

#[test]
fn colorize_wraps_strings() {
    let tokens = tokenize("SELECT 'hello' FROM t;");
    let formatted = beautify(&tokens);
    let colored = colorize(&formatted, &Palette::ansi());
    assert!(colored.contains("\x1b[32m'hello'\x1b[0m"));
}

#[test]
fn colorize_wraps_numbers() {
    let tokens = tokenize("SELECT 42;");
    let formatted = beautify(&tokens);
    let colored = colorize(&formatted, &Palette::ansi());
    assert!(colored.contains("\x1b[33m42\x1b[0m"));
}

#[test]
fn colorize_wraps_comments() {
    let tokens = tokenize("-- a note\nSELECT 1;");
    let formatted = beautify(&tokens);
    let colored = colorize(&formatted, &Palette::ansi());
    assert!(colored.contains("\x1b[2m-- a note\x1b[0m"));
}
