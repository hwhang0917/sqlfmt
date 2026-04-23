# sqlfmt

A fast SQL formatter and minifier. Single binary, no runtime dependencies.

## Installation

### Quick Install (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/hwhang0917/sqlfmt/main/scripts/install.sh | sh
```

This downloads the latest release binary and installs it to `~/.local/bin`. Set `SQLFMT_INSTALL_DIR` to customize the install location.

### Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/hwhang0917/sqlfmt/main/scripts/uninstall.sh | sh
```

### Build from Source

Requires [Rust](https://www.rust-lang.org/tools/install) toolchain.

```bash
git clone https://github.com/hwhang0917/sqlfmt.git
cd sqlfmt
cargo build --release
cp target/release/sqlfmt ~/.local/bin/
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
  -U, --update         Update sqlfmt to the latest release
  -h, --help           Print help
  -V, --version        Print version
```

Output is syntax-highlighted when stdout is a terminal and suppressed when
piped or redirected. `NO_COLOR` is honored.

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
