use sqlfmt::formatter;
use sqlfmt::tokenizer;

use std::io::{self, IsTerminal, Read};
use std::process;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ColorWhen {
    Auto,
    Always,
    Never,
}

struct Cli {
    sql: Option<String>,
    minify: bool,
    color: ColorWhen,
}

enum Action {
    Run(Cli),
    Help,
    Version,
}

const HELP_TEMPLATE: &str = "\
sqlfmt {version}
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
";

fn print_help() {
    print!("{}", HELP_TEMPLATE.replace("{version}", env!("CARGO_PKG_VERSION")));
}

fn print_version() {
    println!("sqlfmt {}", env!("CARGO_PKG_VERSION"));
}

fn print_usage_error(msg: &str) {
    eprintln!("sqlfmt: {msg}");
    eprintln!("Usage: sqlfmt [OPTIONS] [SQL]");
    eprintln!("For more information, try '--help'.");
}

fn parse_color(value: &str) -> Result<ColorWhen, String> {
    match value {
        "auto" => Ok(ColorWhen::Auto),
        "always" => Ok(ColorWhen::Always),
        "never" => Ok(ColorWhen::Never),
        other => Err(format!(
            "invalid value '{other}' for '--color' [possible values: auto, always, never]"
        )),
    }
}

fn parse_args<I: Iterator<Item = String>>(args: I) -> Result<Action, String> {
    let mut sql: Option<String> = None;
    let mut minify = false;
    let mut color = ColorWhen::Auto;
    let mut iter = args;
    let mut positional_only = false;

    while let Some(arg) = iter.next() {
        if positional_only {
            if sql.is_some() {
                return Err(format!("unexpected argument '{arg}'"));
            }
            sql = Some(arg);
            continue;
        }

        match arg.as_str() {
            "--" => positional_only = true,
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "-m" | "--minify" => minify = true,
            "--color" => {
                let value = iter.next().ok_or_else(|| {
                    "a value is required for '--color <WHEN>' but none was supplied".to_string()
                })?;
                color = parse_color(&value)?;
            }
            s if s.starts_with("--color=") => {
                color = parse_color(&s["--color=".len()..])?;
            }
            s if s.starts_with("--") || (s.starts_with('-') && s.len() > 1) => {
                return Err(format!("unexpected argument '{s}'"));
            }
            _ => {
                if sql.is_some() {
                    return Err(format!("unexpected argument '{arg}'"));
                }
                sql = Some(arg);
            }
        }
    }

    Ok(Action::Run(Cli { sql, minify, color }))
}

fn should_colorize(when: ColorWhen) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => {
            std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
        }
    }
}

fn main() {
    let cli = match parse_args(std::env::args().skip(1)) {
        Ok(Action::Run(cli)) => cli,
        Ok(Action::Help) => {
            print_help();
            process::exit(0);
        }
        Ok(Action::Version) => {
            print_version();
            process::exit(0);
        }
        Err(msg) => {
            print_usage_error(&msg);
            process::exit(2);
        }
    };

    let input = match cli.sql {
        Some(sql) => sql,
        None => {
            if io::stdin().is_terminal() {
                print_help();
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
    let formatted = if cli.minify {
        formatter::minify(&tokens)
    } else {
        formatter::beautify(&tokens)
    };

    let output = if should_colorize(cli.color) {
        formatter::colorize(&formatted, &formatter::Palette::ansi())
    } else {
        formatted
    };

    println!("{output}");
}
