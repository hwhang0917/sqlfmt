# sqlfmt Design

A single-binary CLI tool that formats and beautifies SQL, or minifies it.

## Usage

```
cat file.sql | sqlfmt          # beautify via stdin
cat file.sql | sqlfmt -m       # minify via stdin
sqlfmt 'SELECT * FROM table;'  # beautify string arg
sqlfmt -m 'SELECT 1;'          # minify string arg
sqlfmt -h                      # help
```

## Decisions

- **Language:** Rust — single binary, fast, no runtime
- **SQL dialect:** ANSI standard SQL only
- **Parsing strategy:** Hand-written tokenizer (no AST). Best-effort: unrecognized tokens pass through unchanged.
- **Input modes:** stdin (when piped) or positional string argument. No file path arguments.
- **Dependencies:** `clap` for CLI arg parsing. Tokenizer and formatter are hand-written with zero deps.

## Architecture

Three modules:

- `tokenizer` — lexes raw SQL into a token stream
- `formatter` — walks tokens and emits formatted output (beautify or minify)
- `main` — CLI parsing, input detection, orchestration

## Token Types

| Token          | Examples                              |
|----------------|---------------------------------------|
| Keyword        | SELECT, FROM, WHERE, JOIN, INSERT ... |
| Identifier     | table_name, "quoted_id"               |
| StringLiteral  | 'hello world'                         |
| NumberLiteral  | 42, 3.14                              |
| Operator       | =, <, >, <=, >=, <>, !=, +, -, *, /  |
| Comma          | ,                                     |
| Semicolon      | ;                                     |
| OpenParen      | (                                     |
| CloseParen     | )                                     |
| Comment        | -- line, /* block */                  |
| Whitespace     | spaces, tabs, newlines                |
| Other          | anything unrecognized                 |

## Formatting Rules — Beautify

- Major clauses start on a new line at current indent level:
  SELECT, FROM, WHERE, GROUP BY, ORDER BY, HAVING, LIMIT,
  JOIN, LEFT JOIN, INNER JOIN, RIGHT JOIN, OUTER JOIN, CROSS JOIN,
  INSERT INTO, UPDATE, SET, DELETE FROM, VALUES,
  UNION, EXCEPT, INTERSECT, ON
- Content after a major clause keyword is indented one level
- AND / OR in WHERE clauses start on a new line at clause indent
- Subqueries inside parentheses increase indent level
- Commas: trailing style — comma at end of line, next item on new line
- Semicolons end a statement; next statement gets a blank line separator
- Comments preserved in place
- Indent unit: 2 spaces
- Keywords uppercased in output

## Formatting Rules — Minify

- Strip all comments
- Collapse all whitespace to single spaces
- Remove unnecessary whitespace around operators and parens
- Each statement separated by `;` with no extra whitespace
- Output is single-line per statement

## CLI Interface

```
sqlfmt [OPTIONS] [SQL]

Arguments:
  [SQL]  SQL string to format (reads from stdin if omitted)

Options:
  -m, --minify  Minify SQL instead of beautifying
  -h, --help    Print help
```

Input priority:
1. If positional `SQL` arg is provided, format that string
2. If no arg and stdin is not a TTY, read from stdin
3. If no arg and stdin is a TTY, print help and exit with error

## Error Handling

- Best-effort: unrecognized tokens pass through unchanged
- UTF-8 input errors: print error to stderr, exit code 1
- Empty input: empty output, exit code 0
