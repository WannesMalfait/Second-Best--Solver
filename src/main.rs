use std::io::Write;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
#[command(author, version, about, multicall = true)]
enum Command {
    /// Quit the CLI
    #[command(alias("exit"))]
    Quit,
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let mut buf;
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        buf = "".to_string();
        std::io::stdin().read_line(&mut buf).unwrap();
        let args = match Cli::try_parse_from(buf.split_ascii_whitespace()) {
            Ok(args) => args,
            Err(e) => {
                println!();
                e.print().unwrap();
                continue;
            }
        };
        match args.command {
            Command::Quit => return,
        }
    }
}
