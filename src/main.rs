use std::io::Write;

use cli::Cli;

mod cli;
pub mod movegen;
pub mod position;
mod solver;

fn main() {
    let mut buf;
    let mut cli = Cli::default();
    println!(
        "A solver for \"Second-Best!\" by Wannes Malfait.\nType `help` for usage information."
    );
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        buf = "".to_string();
        std::io::stdin().read_line(&mut buf).unwrap();
        match cli.execute_command(&buf) {
            Ok(quit) => {
                if quit {
                    return;
                }
            }
            Err(e) => return eprintln!("{e}"),
        }
    }
}
