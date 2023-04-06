use clap::{Args, Parser, Subcommand};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;

use crate::position::{MoveFailed, Position};
use crate::solver::Solver;
use crate::{bench, eval};

#[derive(Subcommand, Debug, PartialEq, Eq)]
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
    /// Evaluate the current position to the given depth.
    Eval {
        #[arg(default_value_t = 5)]
        /// The depth to which to evaluate the given position.
        depth: usize,
    },
    /// Generate a benchmark file with the given parameters
    GenBench(GenBenchArgs),
    /// Run benchmarks
    Bench {
        /// The number of threads to run the benchmarks on.
        num_threads: usize,
    },
    /// Stop any currently running searches.
    Stop,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct GenBenchArgs {
    /// The number of positions to generate.
    num_positions: usize,
    /// The minimal number of moves in each position.
    min_moves: usize,
    /// The maximal number of moves in each position.
    max_moves: usize,
    /// The minimal amount of depth needed to solve each position.
    min_depth: usize,
    /// The maximal amount of depth needed to solve each position.
    max_depth: usize,
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

struct SearchRequest {
    solver: Arc<Mutex<Solver>>,
    depth: usize,
}

struct GenBenchRequest {
    abort: Arc<AtomicBool>,
    bench_args: GenBenchArgs,
}

struct RunBenchRequest {
    abort: Arc<AtomicBool>,
    num_threads: usize,
}

enum ThreadRequest {
    Search(SearchRequest),
    GenBench(GenBenchRequest),
    RunBench(RunBenchRequest),
    Quit,
}

/// A structure for parsing command line arguments
/// and then executing them.
/// Search is run in the background, so that new
/// commands can be received while running.
pub struct Cli {
    solver: Arc<Mutex<Solver>>,
    abort: Arc<AtomicBool>,
    sender: Sender<ThreadRequest>,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        let abort = Arc::new(AtomicBool::new(false));
        let solver = Arc::new(Mutex::new(Solver::new(abort.clone())));
        let (tx, rx) = mpsc::channel::<ThreadRequest>();
        std::thread::Builder::new()
            .name("Receiver".to_string())
            .stack_size(5_000_000)
            .spawn(move || loop {
                if let Ok(request) = rx.recv() {
                    match request {
                        ThreadRequest::Quit => return,
                        ThreadRequest::Search(req) => {
                            let mut solver = req.solver.lock().unwrap();
                            solver.be_noisy();
                            let eval = solver.search(req.depth);
                            solver.be_quiet();
                            println!(
                                "{}",
                                eval::explain_eval(
                                    solver.position.num_moves() as isize,
                                    solver.position.current_player(),
                                    eval
                                )
                            );
                        }
                        ThreadRequest::GenBench(GenBenchRequest {
                            abort,
                            bench_args:
                                GenBenchArgs {
                                    num_positions,
                                    min_moves,
                                    max_moves,
                                    min_depth,
                                    max_depth,
                                },
                        }) => {
                            bench::generate_benchmark_file(
                                abort,
                                num_positions,
                                min_moves..max_moves,
                                min_depth..max_depth,
                            )
                            .unwrap();
                        }
                        ThreadRequest::RunBench(RunBenchRequest { abort, num_threads }) => {
                            bench::run_benchmarks(abort, num_threads).unwrap();
                        }
                    }
                }
            })
            .unwrap();
        Self {
            solver,
            abort,
            sender: tx,
        }
    }

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
            Command::Quit => {
                self.abort.store(true, Ordering::Relaxed);
                self.sender.send(ThreadRequest::Quit).unwrap();
                return Ok(true);
            }
            Command::Show => self.solver.lock().unwrap().position.show(),
            Command::SetPos { moves } => {
                let solver = &mut *self.solver.lock().unwrap();
                solver.position = Position::default();
                if let Err(e) = solver.position.parse_and_play_moves(moves) {
                    Self::display_error_help(e);
                } else {
                    solver.position.show();
                }
            }
            Command::Play { moves } => {
                self.abort.store(false, Ordering::Relaxed);
                let solver = &mut *self.solver.lock().unwrap();
                if let Err(e) = solver.position.parse_and_play_moves(moves) {
                    Self::display_error_help(e);
                } else {
                    solver.position.show();
                }
            }
            Command::Eval { depth } => {
                let solver = self.solver.clone();
                let req = SearchRequest { solver, depth };
                self.sender.send(ThreadRequest::Search(req)).unwrap();
            }
            Command::GenBench(gen_bench_args) => {
                self.abort.store(false, Ordering::Relaxed);
                let req = GenBenchRequest {
                    abort: self.abort.clone(),
                    bench_args: gen_bench_args,
                };
                self.sender.send(ThreadRequest::GenBench(req)).unwrap();
            }
            Command::Bench {
                num_threads: threads,
            } => {
                self.abort.store(false, Ordering::Relaxed);
                let req = RunBenchRequest {
                    abort: self.abort.clone(),
                    num_threads: threads,
                };
                self.sender.send(ThreadRequest::RunBench(req)).unwrap();
            }
            Command::Stop => {
                self.abort.store(true, Ordering::Relaxed);
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
            MoveFailed::PositionWinning => {
                println!("The position is won for the current player, so the game is over.")
            }
        }
    }
}
