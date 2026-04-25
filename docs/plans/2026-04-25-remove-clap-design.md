# Remove clap dependency — Design

## Goal

Make `sqlfmt` a truly zero-dependency binary by replacing `clap` with a hand-written argument parser using only the Rust standard library. The user-visible CLI surface is unchanged.

## Constraints

The existing CLI tests in `tests/cli_tests.rs` define the contract and must pass unchanged:

- positional `sql` arg, optional, falls back to stdin
- `-m` / `--minify` flag
- `--color <auto|always|never>` (default: `auto`)
- `-h` / `--help` prints help, exits 0; help text contains `sqlfmt`, `--minify`, `--color`
- `-V` / `--version` prints version, exits 0; output contains `sqlfmt` and `CARGO_PKG_VERSION`
- empty input exits 0 with empty output
- TTY + no positional + no stdin pipe → print help, exit 1 (preserve current behavior)

## Approach

A small parser inside `src/main.rs` — no new module. Roughly 80 LOC, replacing the `clap::Parser` derive while leaving the rest of `main` untouched.

### Cargo.toml

Drop `clap` from `[dependencies]`. The dependencies table becomes empty (or is removed entirely). `Cargo.lock` is regenerated to remove `clap` and its transitives.

### Argument grammar

Iterate `std::env::args().skip(1)` once, left to right.

- `--` ends option parsing; remaining tokens are positional.
- Long flags:
  - `--minify`
  - `--help`
  - `--version`
  - `--color VAL` (next arg is the value)
  - `--color=VAL` (inline value)
- Short flags:
  - `-m`
  - `-h`
  - `-V`
  - No bundling (`-mh` is rejected). Each short flag is a distinct semantic; bundling has no current use.
- Positional: at most one (the SQL string). A second positional is an error.
- Unknown flag → error.

### Help text

Hand-written, mirrors clap's `USAGE/ARGS/OPTIONS` style so that human readers and the test assertions both stay happy:

```
sqlfmt <CARGO_PKG_VERSION>
Format and beautify SQL

USAGE:
    sqlfmt [OPTIONS] [SQL]

ARGS:
    [SQL]    SQL string to format (reads from stdin if omitted)

OPTIONS:
    -m, --minify          Minify SQL instead of beautifying
        --color <WHEN>    When to use ANSI color output [auto|always|never] (default: auto)
    -h, --help            Print help
    -V, --version         Print version
```

### Version text

`sqlfmt <CARGO_PKG_VERSION>\n` to stdout.

### Error path

Parse errors (unknown flag, missing value for `--color`, invalid `--color` value, extra positional) print to stderr and exit 2:

```
sqlfmt: <message>
Usage: sqlfmt [OPTIONS] [SQL]
For more information, try '--help'.
```

Exit code 2 matches the POSIX/getopt convention for usage errors and clap's default.

### Internal shape

The `Cli` struct keeps the same fields as today so the body of `main` does not change:

```rust
struct Cli {
    sql: Option<String>,
    minify: bool,
    color: ColorWhen,
}

enum ColorWhen { Auto, Always, Never }
```

A single `parse_args(args: impl Iterator<Item = String>) -> Result<Action, ParseError>` function returns either `Action::Run(Cli)`, `Action::Help`, or `Action::Version`. `main` matches on the result, prints help/version when appropriate, exits 2 on error, and otherwise proceeds exactly as it does today.

The TTY-no-arg-no-pipe branch in `main` switches from `Cli::parse_from(["sqlfmt", "--help"])` to a direct call to the new `print_help()` function followed by `process::exit(1)`.

## Testing

All existing tests in `tests/cli_tests.rs` must pass unchanged. No new tests are required by the spec, but the implementation plan may add coverage for:

- `--color=never` (inline value form)
- unknown flag → exit code 2
- extra positional argument → exit code 2

## Out of scope

- No change to `tokenizer`, `formatter`, or library surface.
- No change to `lib.rs`.
- No change to README, release scripts, or CI.
- No bundling of short flags, no abbreviated long flags, no `--no-minify` style negations.
