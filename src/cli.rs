use clap::{Parser, Subcommand};
use std::vec::Vec;

use crate::position::{MoveFailed, Position};

#[derive(Subcommand, Debug)]
#[command(author, version, about, multicall = true)]
enum Command {
    /// Quit the CLI
    #[command(alias("exit"))]
    Quit,
    /// Print the current position
    #[command(alias("display"))]
    Show,
    /// Set the position using a sequence of moves
    SetPos {
        /// The moves to be played from the starting position
        moves: Vec<String>,
    },
    /// Play a sequence of moves from the current position
    Play {
        /// The moves to be played from the current position.
        moves: Vec<String>,
    },
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Default)]
pub struct Cli {
    pos: Position,
}

impl Cli {
    /// Parses and executes the command.
    /// On success: returns whether to quit the cli or not.
    /// On failure: returns the io error that caused a failure.
    ///
    /// NOTE: invalid commands are not returned as errors, since
    /// these are communicated with the user through the cli
    pub fn execute_command(&mut self, command: &str) -> Result<bool, std::io::Error> {
        let args = match CliArgs::try_parse_from(command.split_ascii_whitespace()) {
            Ok(args) => args,
            Err(e) => {
                println!();
                e.print()?;
                // Parse error is bad input from user, but not an actual problem.
                return Ok(false);
            }
        };
        match args.command {
            Command::Quit => return Ok(true),
            Command::Show => self.pos.show(),
            Command::SetPos { moves } => {
                self.pos = Position::default();
                if let Err(e) = self.pos.parse_and_play_moves(moves) {
                    Self::display_error_help(e);
                } else {
                    self.pos.show();
                }
            }
            Command::Play { moves } => {
                if let Err(e) = self.pos.parse_and_play_moves(moves) {
                    Self::display_error_help(e);
                } else {
                    self.pos.show();
                }
            }
        }
        Ok(false)
    }

    fn display_error_help(error: MoveFailed) {
        match error {
            MoveFailed::InvalidFromSpot => println!("Invalid \"from\" spot in the given move."),
            MoveFailed::InvalidToSpot => println!("Invalid \"to\" spot in the given move."),
            MoveFailed::InvalidSecondBest => {
                println!("Second best can not be called anymore on this move.")
            }
            MoveFailed::MissingFromSpot => {
                println!("The \"from\" spot was not given for the given move.")
            }
            MoveFailed::MoveBanned => println!(
                "The given move can not be played anymore, since \"Second Best!\" was called."
            ),
            MoveFailed::SameFromAndTo => {
                println!("The \"from\" and \"to\" spot in the given move are the same.")
            }
            MoveFailed::ParseError => {
                println!(
                    "The given move could not be parsed into a move.\n\
                It should be either a '!' (representing a \"Second Best!\" call),\n\
                a single number indicating the stack to move to, \n\
                or two numbers separated by a '-' indicating the stacks to move from and to."
                )
            }
        }
    }
}
