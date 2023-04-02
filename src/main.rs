use second_best::cli::Cli;

fn main() {
    let mut buf;
    let mut cli = Cli::default();
    println!(
        "A solver for \"Second-Best!\" by Wannes Malfait.\nType `help` for usage information."
    );
    loop {
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
