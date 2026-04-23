mod update;

use sqlfmt::formatter;
use sqlfmt::tokenizer;

use clap::{Parser, ValueEnum};
use std::io::{self, IsTerminal, Read};
use std::process;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum ColorWhen {
    Auto,
    Always,
    Never,
}

#[derive(Parser)]
#[command(name = "sqlfmt", about = "Format and beautify SQL", version)]
struct Cli {
    /// SQL string to format (reads from stdin if omitted)
    sql: Option<String>,

    /// Minify SQL instead of beautifying
    #[arg(short, long)]
    minify: bool,

    /// When to use ANSI color output
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto)]
    color: ColorWhen,

    /// Update sqlfmt to the latest release
    #[arg(short = 'U', long)]
    update: bool,
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
    let cli = Cli::parse();

    if cli.update {
        update::run();
        return;
    }

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
