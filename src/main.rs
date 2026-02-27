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
