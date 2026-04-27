# sqlfmt

![Crates.io Version](https://img.shields.io/crates/v/rf-sqlfmt) ![Crates.io Total Downloads](https://img.shields.io/crates/d/rf-sqlfmt) ![GitHub License](https://img.shields.io/github/license/hwhang0917/sqlfmt)

A fast SQL formatter and minifier. Single binary, no runtime dependencies.

## Installation

Requires a [Rust toolchain](https://www.rust-lang.org/tools/install).

```bash
cargo install rf-sqlfmt
```

The binary is installed to `~/.cargo/bin/sqlfmt`.

### Build from source

```bash
git clone https://github.com/hwhang0917/sqlfmt.git
cd sqlfmt
cargo install --path .
```

## Usage

```bash
# Beautify SQL from stdin
cat query.sql | sqlfmt

# Beautify SQL from string argument
sqlfmt 'SELECT id, name FROM users WHERE id = 1;'

# Minify SQL
cat query.sql | sqlfmt -m
sqlfmt -m 'SELECT id, name FROM users WHERE id = 1;'
```

### Options

```
sqlfmt [OPTIONS] [SQL]

Arguments:
  [SQL]  SQL string to format (reads from stdin if omitted)

Options:
  -m, --minify         Minify SQL instead of beautifying
      --color <WHEN>   When to use ANSI color output [auto|always|never] (default: auto)
  -h, --help           Print help
  -V, --version        Print version
```

Output is syntax-highlighted when stdout is a terminal and suppressed when
piped or redirected. `NO_COLOR` is honored.

> **Note:** SQL line comments start with `--`, which collides with flag
> parsing. To pass a SQL string that begins with `--`, end option parsing
> with a literal `--` first, or pipe via stdin:
>
> ```bash
> sqlfmt -- '-- a comment; SELECT * FROM users;'
> echo '-- a comment; SELECT * FROM users;' | sqlfmt
> ```

## Examples

### Beautify

```sql
-- Input
SELECT id, name, email FROM users WHERE active = 1 AND role = 'admin' ORDER BY name;

-- Output
SELECT
  id,
  name,
  email
FROM
  users
WHERE
  active = 1
  AND role = 'admin'
ORDER BY
  name;
```

### DDL

Column definitions are broken onto separate lines. Backtick (MySQL/MariaDB/
SQLite), double-quote (ANSI/PostgreSQL), and bracket (MSSQL) quoted
identifiers are recognized.

```sql
-- Input
CREATE TABLE `user` (`id` INTEGER PRIMARY KEY NOT NULL, `name` TEXT NOT NULL, `age` INTEGER DEFAULT -1 NOT NULL);

-- Output
CREATE TABLE `user` (
  `id` INTEGER PRIMARY KEY NOT NULL,
  `name` TEXT NOT NULL,
  `age` INTEGER DEFAULT -1 NOT NULL
);
```

### Minify

```sql
-- Input
SELECT
  id,
  name
FROM
  users
WHERE
  id = 1;

-- Output
SELECT id,name FROM users WHERE id=1;
```

## License

[MIT](LICENSE)
