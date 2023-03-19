use std::io::Write;

use cli::Cli;

mod cli;
pub mod position;

fn main() {
    let mut buf;
    let mut cli = Cli::default();
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
