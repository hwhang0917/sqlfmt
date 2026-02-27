# sqlfmt

A fast SQL formatter and minifier. Single binary, no runtime dependencies.

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
  -m, --minify   Minify SQL instead of beautifying
  -h, --help     Print help
  -V, --version  Print version
```

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

## Build

```bash
cargo build --release
```

Binary will be at `target/release/sqlfmt`.

## License

[MIT](LICENSE)
