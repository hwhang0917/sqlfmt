# sqlfmt Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a single-binary Rust CLI tool that beautifies or minifies SQL from stdin or a string argument.

**Architecture:** Three modules — `tokenizer` (lexer), `formatter` (beautify/minify), `main` (CLI + I/O). Hand-written tokenizer produces a token stream; formatter walks it to produce output.

**Tech Stack:** Rust, clap 4.5 (derive)

---

### Task 1: Scaffold Rust project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/tokenizer.rs`
- Create: `src/formatter.rs`

**Step 1: Initialize cargo project**

Run: `cargo init --name sqlfmt`
Expected: Creates `Cargo.toml` and `src/main.rs`

**Step 2: Add clap dependency**

Run: `cargo add clap --features derive`
Expected: clap added to `Cargo.toml` dependencies

**Step 3: Create module files**

Create empty `src/tokenizer.rs` and `src/formatter.rs`.

**Step 4: Set up main.rs with CLI parsing and module declarations**

```rust
mod formatter;
mod tokenizer;

use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::process;

#[derive(Parser)]
#[command(name = "sqlfmt", about = "Format and beautify SQL")]
struct Cli {
    /// SQL string to format (reads from stdin if omitted)
    sql: Option<String>,

    /// Minify SQL instead of beautifying
    #[arg(short, long)]
    minify: bool,
}

fn main() {
    let cli = Cli::parse();

    let input = match cli.sql {
        Some(sql) => sql,
        None => {
            if io::stdin().is_terminal() {
                Cli::parse_from(["sqlfmt", "--help"]);
                process::exit(1);
            }
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("sqlfmt: {e}");
                process::exit(1);
            }
            buf
        }
    };

    if input.is_empty() {
        return;
    }

    let tokens = tokenizer::tokenize(&input);
    let output = if cli.minify {
        formatter::minify(&tokens)
    } else {
        formatter::beautify(&tokens)
    };

    print!("{output}");
}
```

Add stubs to `src/tokenizer.rs`:

```rust
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
```

Add stubs to `src/formatter.rs`:

```rust
use crate::tokenizer::Token;

pub fn beautify(_tokens: &[Token]) -> String {
    String::new()
}

pub fn minify(_tokens: &[Token]) -> String {
    String::new()
}
```

**Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors

**Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/
git commit -m "feat: scaffold sqlfmt project with CLI parsing"
```

---

### Task 2: Tokenizer — basic tokens

**Files:**
- Modify: `src/tokenizer.rs`
- Create: `tests/tokenizer_tests.rs`

**Step 1: Write failing tests for basic tokenization**

Create `tests/tokenizer_tests.rs`:

```rust
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
```

**Step 2: Make tokenizer a library module so tests can import it**

Add to `src/lib.rs`:

```rust
pub mod tokenizer;
pub mod formatter;
```

Update `src/main.rs` to use the library:

```rust
use clap::Parser;
use sqlfmt::formatter;
use sqlfmt::tokenizer;
use std::io::{self, IsTerminal, Read};
use std::process;
```

Remove `mod` declarations from `main.rs`.

**Step 3: Run tests to verify they fail**

Run: `cargo test`
Expected: Tests fail (tokenize returns empty vec)

**Step 4: Implement the tokenizer**

Replace the body of `tokenize` in `src/tokenizer.rs` with:

```rust
const KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER",
    "CROSS", "FULL", "ON", "AND", "OR", "NOT", "IN", "IS", "NULL", "AS",
    "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "INSERT", "INTO",
    "VALUES", "UPDATE", "SET", "DELETE", "CREATE", "TABLE", "DROP", "ALTER",
    "INDEX", "VIEW", "UNION", "ALL", "DISTINCT", "EXCEPT", "INTERSECT",
    "EXISTS", "BETWEEN", "LIKE", "CASE", "WHEN", "THEN", "ELSE", "END",
    "ASC", "DESC", "TRUE", "FALSE", "CAST", "WITH", "RECURSIVE", "PRIMARY",
    "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK",
    "UNIQUE", "IF", "REPLACE", "TEMPORARY", "TEMP", "RETURNING",
    "NATURAL", "USING", "FETCH", "NEXT", "ROWS", "ONLY", "FIRST",
    "NULLS", "LAST", "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE",
    "OVER", "PARTITION", "ROW_NUMBER", "RANK", "DENSE_RANK", "LAG", "LEAD",
    "WINDOW", "RANGE", "UNBOUNDED", "PRECEDING", "FOLLOWING", "CURRENT",
    "ROW", "GRANT", "REVOKE", "ROLLBACK", "COMMIT", "BEGIN", "TRANSACTION",
    "SAVEPOINT", "RELEASE", "TRIGGER", "EXECUTE", "PROCEDURE", "FUNCTION",
    "DECLARE", "CURSOR", "OPEN", "CLOSE",
];

fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word.to_uppercase().as_str())
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Whitespace
        if ch.is_ascii_whitespace() {
            let start = i;
            while i < len && chars[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(Token::Whitespace(chars[start..i].iter().collect()));
            continue;
        }

        // Line comment
        if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
            let start = i;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            tokens.push(Token::Comment(chars[start..i].iter().collect()));
            continue;
        }

        // Block comment
        if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // skip */
            }
            tokens.push(Token::Comment(chars[start..i].iter().collect()));
            continue;
        }

        // String literal
        if ch == '\'' {
            let start = i;
            i += 1;
            while i < len {
                if chars[i] == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        i += 2; // escaped quote
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            tokens.push(Token::StringLiteral(chars[start..i].iter().collect()));
            continue;
        }

        // Quoted identifier
        if ch == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                i += 1;
            }
            if i < len {
                i += 1; // closing quote
            }
            tokens.push(Token::Identifier(chars[start..i].iter().collect()));
            continue;
        }

        // Number
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token::NumberLiteral(chars[start..i].iter().collect()));
            continue;
        }

        // Punctuation
        match ch {
            '(' => { tokens.push(Token::OpenParen); i += 1; continue; }
            ')' => { tokens.push(Token::CloseParen); i += 1; continue; }
            ',' => { tokens.push(Token::Comma); i += 1; continue; }
            ';' => { tokens.push(Token::Semicolon); i += 1; continue; }
            _ => {}
        }

        // Multi-char operators
        if i + 1 < len {
            let two: String = chars[i..i+2].iter().collect();
            if matches!(two.as_str(), "<>" | "<=" | ">=" | "!=" | "||") {
                tokens.push(Token::Operator(two));
                i += 2;
                continue;
            }
        }

        // Single-char operators
        if matches!(ch, '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~') {
            tokens.push(Token::Operator(ch.to_string()));
            i += 1;
            continue;
        }

        // Word (keyword or identifier)
        if ch.is_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if is_keyword(&word) {
                tokens.push(Token::Keyword(word));
            } else {
                tokens.push(Token::Identifier(word));
            }
            continue;
        }

        // Dot (for qualified names like table.column)
        if ch == '.' {
            tokens.push(Token::Operator(".".to_string()));
            i += 1;
            continue;
        }

        // Other — pass through
        tokens.push(Token::Other(ch.to_string()));
        i += 1;
    }

    tokens
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: All tokenizer tests pass

**Step 6: Commit**

```bash
git add src/ tests/ Cargo.toml
git commit -m "feat: implement SQL tokenizer with tests"
```

---

### Task 3: Formatter — minify mode

**Files:**
- Modify: `src/formatter.rs`
- Create: `tests/minify_tests.rs`

**Step 1: Write failing tests for minify**

Create `tests/minify_tests.rs`:

```rust
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test minify`
Expected: Tests fail

**Step 3: Implement minify**

Replace the `minify` function in `src/formatter.rs`:

```rust
use crate::tokenizer::Token;

pub fn minify(tokens: &[Token]) -> String {
    let mut out = String::new();
    let mut prev_needs_space = false;

    for token in tokens {
        match token {
            Token::Whitespace(_) | Token::Comment(_) => {
                // Skip — but mark that we might need a space
                if !out.is_empty() {
                    prev_needs_space = true;
                }
            }
            Token::OpenParen => {
                prev_needs_space = false;
                out.push('(');
            }
            Token::CloseParen => {
                prev_needs_space = false;
                out.push(')');
            }
            Token::Comma => {
                prev_needs_space = false;
                out.push(',');
            }
            Token::Semicolon => {
                prev_needs_space = false;
                out.push(';');
            }
            Token::Operator(op) => {
                prev_needs_space = false;
                out.push_str(op);
            }
            Token::Keyword(kw) => {
                if prev_needs_space {
                    out.push(' ');
                }
                prev_needs_space = false;
                out.push_str(&kw.to_uppercase());
            }
            Token::Identifier(id) => {
                if prev_needs_space {
                    out.push(' ');
                }
                prev_needs_space = false;
                out.push_str(id);
            }
            Token::StringLiteral(s) => {
                if prev_needs_space {
                    out.push(' ');
                }
                prev_needs_space = false;
                out.push_str(s);
            }
            Token::NumberLiteral(n) => {
                if prev_needs_space {
                    out.push(' ');
                }
                prev_needs_space = false;
                out.push_str(n);
            }
            Token::Other(o) => {
                if prev_needs_space {
                    out.push(' ');
                }
                prev_needs_space = false;
                out.push_str(o);
            }
        }
    }

    out.trim_end().to_string()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test minify`
Expected: All minify tests pass

**Step 5: Commit**

```bash
git add src/formatter.rs tests/minify_tests.rs
git commit -m "feat: implement SQL minify formatter"
```

---

### Task 4: Formatter — beautify mode

**Files:**
- Modify: `src/formatter.rs`
- Create: `tests/beautify_tests.rs`

**Step 1: Write failing tests for beautify**

Create `tests/beautify_tests.rs`:

```rust
use sqlfmt::tokenizer::tokenize;
use sqlfmt::formatter::beautify;

#[test]
fn beautify_simple_select() {
    let tokens = tokenize("SELECT id, name FROM users WHERE id = 1;");
    let expected = "\
SELECT
  id,
  name
FROM
  users
WHERE
  id = 1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_join() {
    let tokens = tokenize("SELECT a.id FROM a JOIN b ON a.id = b.id;");
    let expected = "\
SELECT
  a.id
FROM
  a
JOIN
  b
ON
  a.id = b.id;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_and_or() {
    let tokens = tokenize("SELECT 1 FROM t WHERE a = 1 AND b = 2 OR c = 3;");
    let expected = "\
SELECT
  1
FROM
  t
WHERE
  a = 1
  AND b = 2
  OR c = 3;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_multiple_statements() {
    let tokens = tokenize("SELECT 1; SELECT 2;");
    let expected = "\
SELECT
  1;

SELECT
  2;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_subquery() {
    let tokens = tokenize("SELECT * FROM (SELECT id FROM t);");
    let expected = "\
SELECT
  *
FROM
  (
    SELECT
      id
    FROM
      t
  );
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_preserves_comments() {
    let tokens = tokenize("-- header\nSELECT 1;");
    let expected = "\
-- header
SELECT
  1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_keywords_uppercased() {
    let tokens = tokenize("select id from users where id = 1;");
    let expected = "\
SELECT
  id
FROM
  users
WHERE
  id = 1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_insert() {
    let tokens = tokenize("INSERT INTO users (id, name) VALUES (1, 'Alice');");
    let expected = "\
INSERT INTO
  users (id, name)
VALUES
  (1, 'Alice');
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_group_by_order_by() {
    let tokens = tokenize("SELECT country, COUNT(*) FROM users GROUP BY country ORDER BY country;");
    let expected = "\
SELECT
  country,
  COUNT(*)
FROM
  users
GROUP BY
  country
ORDER BY
  country;
";
    assert_eq!(beautify(&tokens), expected);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test beautify`
Expected: Tests fail

**Step 3: Implement beautify**

This is the most complex piece. Replace the `beautify` function in `src/formatter.rs`:

```rust
const MAJOR_CLAUSES: &[&str] = &[
    "SELECT", "FROM", "WHERE", "HAVING", "LIMIT", "SET", "VALUES",
    "UNION", "EXCEPT", "INTERSECT",
];

const COMPOUND_CLAUSES: &[(&str, &str)] = &[
    ("GROUP", "BY"),
    ("ORDER", "BY"),
    ("INSERT", "INTO"),
    ("DELETE", "FROM"),
];

const JOIN_KEYWORDS: &[&str] = &[
    "JOIN", "INNER", "LEFT", "RIGHT", "OUTER", "CROSS", "NATURAL", "FULL",
];

fn is_major_clause(kw: &str) -> bool {
    MAJOR_CLAUSES.contains(&kw.to_uppercase().as_str())
}

fn is_join_keyword(kw: &str) -> bool {
    JOIN_KEYWORDS.contains(&kw.to_uppercase().as_str())
}

fn is_compound_first(kw: &str) -> bool {
    COMPOUND_CLAUSES.iter().any(|(first, _)| first.eq_ignore_ascii_case(kw))
}

fn compound_second(first: &str) -> Option<&'static str> {
    COMPOUND_CLAUSES.iter()
        .find(|(f, _)| f.eq_ignore_ascii_case(first))
        .map(|(_, s)| *s)
}

pub fn beautify(tokens: &[Token]) -> String {
    let mut out = String::new();
    let indent_str = "  ";
    let mut indent_level: usize = 0;
    let mut in_clause = false;
    let mut after_keyword = false;
    let mut i = 0;
    let mut paren_depth: usize = 0;
    let mut paren_indent_base: Vec<usize> = Vec::new();
    let mut need_newline = false;
    let mut at_line_start = true;
    let mut after_semicolon = false;

    // Filter out whitespace tokens — we control all spacing
    let significant: Vec<&Token> = tokens.iter()
        .filter(|t| !matches!(t, Token::Whitespace(_)))
        .collect();

    let len = significant.len();

    while i < len {
        let tok = significant[i];

        match tok {
            Token::Keyword(kw) => {
                let upper = kw.to_uppercase();

                // Check for compound clause (GROUP BY, ORDER BY, INSERT INTO, DELETE FROM)
                if is_compound_first(&upper) {
                    if let Some(second) = compound_second(&upper) {
                        // Look ahead for the second keyword
                        if i + 1 < len {
                            if let Token::Keyword(next_kw) = significant[i + 1] {
                                if next_kw.eq_ignore_ascii_case(second) {
                                    // Emit compound clause
                                    if !at_line_start {
                                        out.push('\n');
                                    }
                                    if after_semicolon {
                                        out.push('\n');
                                        after_semicolon = false;
                                    }
                                    for _ in 0..indent_level {
                                        out.push_str(indent_str);
                                    }
                                    out.push_str(&upper);
                                    out.push(' ');
                                    out.push_str(&next_kw.to_uppercase());
                                    in_clause = true;
                                    after_keyword = true;
                                    need_newline = true;
                                    at_line_start = false;
                                    i += 2;
                                    continue;
                                }
                            }
                        }
                    }
                }

                // JOIN-related keywords
                if is_join_keyword(&upper) && upper != "ON" {
                    // Collect full join type (e.g., LEFT JOIN, INNER JOIN)
                    let mut join_parts = vec![upper.clone()];
                    let mut j = i + 1;
                    while j < len {
                        if let Token::Keyword(next_kw) = significant[j] {
                            let next_upper = next_kw.to_uppercase();
                            if is_join_keyword(&next_upper) || next_upper == "JOIN" {
                                join_parts.push(next_upper);
                                j += 1;
                                if join_parts.last().map(|s| s.as_str()) == Some("JOIN") {
                                    break;
                                }
                                continue;
                            }
                        }
                        break;
                    }

                    if join_parts.contains(&"JOIN".to_string()) {
                        if !at_line_start {
                            out.push('\n');
                        }
                        if after_semicolon {
                            out.push('\n');
                            after_semicolon = false;
                        }
                        for _ in 0..indent_level {
                            out.push_str(indent_str);
                        }
                        out.push_str(&join_parts.join(" "));
                        in_clause = true;
                        after_keyword = true;
                        need_newline = true;
                        at_line_start = false;
                        i = j;
                        continue;
                    }
                }

                // ON keyword
                if upper == "ON" {
                    if !at_line_start {
                        out.push('\n');
                    }
                    for _ in 0..indent_level {
                        out.push_str(indent_str);
                    }
                    out.push_str("ON");
                    in_clause = true;
                    after_keyword = true;
                    need_newline = true;
                    at_line_start = false;
                    i += 1;
                    continue;
                }

                // AND / OR
                if upper == "AND" || upper == "OR" {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    out.push_str(&upper);
                    after_keyword = true;
                    at_line_start = false;
                    i += 1;
                    continue;
                }

                // Major clause
                if is_major_clause(&upper) {
                    if !at_line_start {
                        out.push('\n');
                    }
                    if after_semicolon {
                        out.push('\n');
                        after_semicolon = false;
                    }
                    for _ in 0..indent_level {
                        out.push_str(indent_str);
                    }
                    out.push_str(&upper);
                    in_clause = true;
                    after_keyword = true;
                    need_newline = true;
                    at_line_start = false;
                    i += 1;
                    continue;
                }

                // Regular keyword — just emit with spacing
                if need_newline {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    need_newline = false;
                } else if !at_line_start {
                    out.push(' ');
                }
                out.push_str(&upper);
                after_keyword = true;
                at_line_start = false;
            }

            Token::Identifier(id) | Token::NumberLiteral(id) | Token::StringLiteral(id) | Token::Other(id) => {
                if need_newline {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    need_newline = false;
                } else if !at_line_start && !after_dot_or_before_dot(significant.get(i.wrapping_sub(1)), Some(&tok)) {
                    out.push(' ');
                }
                out.push_str(id);
                after_keyword = false;
                at_line_start = false;
            }

            Token::Operator(op) => {
                if op == "." {
                    // No spaces around dot
                    out.push('.');
                } else if need_newline {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    need_newline = false;
                    out.push_str(op);
                } else {
                    if op == "*" && after_keyword {
                        out.push('\n');
                        for _ in 0..indent_level + 1 {
                            out.push_str(indent_str);
                        }
                    } else {
                        out.push(' ');
                    }
                    out.push_str(op);
                }
                after_keyword = false;
                at_line_start = false;
            }

            Token::Comma => {
                out.push(',');
                need_newline = true;
                at_line_start = false;
            }

            Token::Semicolon => {
                out.push(';');
                out.push('\n');
                after_semicolon = true;
                at_line_start = true;
                in_clause = false;
                need_newline = false;
            }

            Token::OpenParen => {
                if need_newline {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    need_newline = false;
                } else if !at_line_start {
                    // Check if previous token is an identifier/keyword (function call)
                    let is_function_call = i > 0 && matches!(
                        significant[i - 1],
                        Token::Keyword(_) | Token::Identifier(_)
                    );
                    if !is_function_call {
                        out.push(' ');
                    }
                }

                // Check if this paren contains a subquery
                let has_subquery = contains_subquery(&significant[i+1..]);

                if has_subquery {
                    out.push_str("(\n");
                    paren_indent_base.push(indent_level);
                    indent_level += 2;
                    for _ in 0..indent_level {
                        out.push_str(indent_str);
                    }
                    paren_depth += 1;
                    at_line_start = false;
                    need_newline = false;
                    // The next token (SELECT etc.) will handle its own newline
                    // We need to signal that we're at the start of a new context
                    at_line_start = true;
                } else {
                    out.push('(');
                    paren_depth += 1;
                    paren_indent_base.push(indent_level);
                }
                after_keyword = false;
                i += 1;
                continue;
            }

            Token::CloseParen => {
                if let Some(base) = paren_indent_base.pop() {
                    if indent_level > base + 1 {
                        // Subquery paren
                        indent_level = base;
                        out.push('\n');
                        for _ in 0..indent_level + 1 {
                            out.push_str(indent_str);
                        }
                        out.push(')');
                    } else {
                        indent_level = base;
                        out.push(')');
                    }
                } else {
                    out.push(')');
                }
                paren_depth = paren_depth.saturating_sub(1);
                need_newline = false;
                at_line_start = false;
            }

            Token::Comment(c) => {
                if need_newline {
                    out.push('\n');
                    for _ in 0..indent_level + 1 {
                        out.push_str(indent_str);
                    }
                    need_newline = false;
                } else if !at_line_start {
                    out.push(' ');
                }
                out.push_str(c);
                if c.starts_with("--") {
                    out.push('\n');
                    at_line_start = true;
                }
            }

            Token::Whitespace(_) => {} // already filtered
        }

        i += 1;
    }

    out
}

fn after_dot_or_before_dot(prev: Option<&&Token>, _current: Option<&&Token>) -> bool {
    if let Some(Token::Operator(op)) = prev {
        if op == "." {
            return true;
        }
    }
    false
}

fn contains_subquery(tokens: &[&Token]) -> bool {
    let mut depth = 0;
    for t in tokens {
        match t {
            Token::OpenParen => depth += 1,
            Token::CloseParen => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            Token::Keyword(kw) if depth == 0 => {
                let upper = kw.to_uppercase();
                if upper == "SELECT" || upper == "INSERT" || upper == "UPDATE" || upper == "DELETE" || upper == "WITH" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test beautify`
Expected: All beautify tests pass

Note: The beautify implementation is the most iterative piece. If tests fail, adjust the formatter logic to match expected output. The test expectations defined above are the source of truth.

**Step 5: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/formatter.rs tests/beautify_tests.rs
git commit -m "feat: implement SQL beautify formatter"
```

---

### Task 5: CLI integration tests

**Files:**
- Create: `tests/cli_tests.rs`

**Step 1: Write CLI integration tests**

Create `tests/cli_tests.rs`:

```rust
use std::process::Command;

fn sqlfmt() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sqlfmt"))
}

#[test]
fn cli_string_arg_beautify() {
    let output = sqlfmt()
        .arg("SELECT * FROM users;")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SELECT"));
    assert!(stdout.contains("FROM"));
}

#[test]
fn cli_string_arg_minify() {
    let output = sqlfmt()
        .args(["-m", "SELECT   *   FROM   users  ;"])
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "SELECT * FROM users;");
}

#[test]
fn cli_stdin_beautify() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"SELECT 1;").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SELECT"));
}

#[test]
fn cli_stdin_minify() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .arg("-m")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"SELECT   1  ;").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "SELECT 1;");
}

#[test]
fn cli_help_flag() {
    let output = sqlfmt()
        .arg("--help")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sqlfmt"));
    assert!(stdout.contains("--minify"));
}

#[test]
fn cli_empty_input() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}
```

**Step 2: Run CLI tests**

Run: `cargo test --test cli_tests`
Expected: All CLI integration tests pass

**Step 3: Commit**

```bash
git add tests/cli_tests.rs
git commit -m "test: add CLI integration tests"
```

---

### Task 6: Final polish and verification

**Files:**
- Modify: `Cargo.toml` (add metadata)

**Step 1: Add project metadata to Cargo.toml**

Add to `[package]` section:

```toml
description = "A fast SQL formatter and minifier"
license = "MIT"
edition = "2021"
```

**Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 3: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/sqlfmt`

**Step 4: Manual smoke test commands**

Run: `echo "SELECT id, name FROM users WHERE id = 1;" | ./target/release/sqlfmt`
Expected: Beautifully formatted output

Run: `echo "SELECT id, name FROM users WHERE id = 1;" | ./target/release/sqlfmt -m`
Expected: `SELECT id,name FROM users WHERE id=1;`

Run: `./target/release/sqlfmt 'SELECT * FROM t;'`
Expected: Formatted output

**Step 5: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add project metadata"
```
