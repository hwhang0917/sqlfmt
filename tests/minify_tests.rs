use sqlfmt::tokenizer::tokenize;
use sqlfmt::formatter::minify;

#[test]
fn minify_simple_select() {
    let tokens = tokenize("  SELECT   *   FROM   users  ;  ");
    assert_eq!(minify(&tokens), "SELECT * FROM users;");
}

#[test]
fn minify_multiline() {
    let tokens = tokenize("SELECT\n  id,\n  name\nFROM\n  users\nWHERE\n  id = 1;");
    assert_eq!(minify(&tokens), "SELECT id,name FROM users WHERE id=1;");
}

#[test]
fn minify_strips_comments() {
    let tokens = tokenize("SELECT -- get all\n* FROM users /* table */;");
    assert_eq!(minify(&tokens), "SELECT * FROM users;");
}

#[test]
fn minify_preserves_strings() {
    let tokens = tokenize("SELECT 'hello   world' FROM t;");
    assert_eq!(minify(&tokens), "SELECT 'hello   world' FROM t;");
}

#[test]
fn minify_multiple_statements() {
    let tokens = tokenize("SELECT 1; SELECT 2;");
    assert_eq!(minify(&tokens), "SELECT 1;SELECT 2;");
}

#[test]
fn minify_parens() {
    let tokens = tokenize("SELECT COUNT( * ) FROM t;");
    assert_eq!(minify(&tokens), "SELECT COUNT(*) FROM t;");
}
