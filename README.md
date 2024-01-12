# Second Best! Solver

A rust program to solve positions in the board game ["Second Best!"](https://jelly2games.com/secondbest).

The official rules:
![Rules of the board game "Second Best!"](https://jelly2games.com/wp-content/themes/jelly2games/img/secondbest_rule_en.png)

## Usage

Clone the repository and then compile and run the code with:

```terminal
cargo run --release
```

You can then interact through a CLI with the solver. Use the `help` command for usage info:

```terminal
help
```

### GUI

It is also possible to use a GUI. For this you need to run

```terminal
cargo run --release --bin gui --features="gui"
```

WARNING: The GUI depends on `bevy_egui`, and might hence require some
dependencies to be installed on linux. See [the bevy docs](https://bevyengine.org/learn/book/getting-started/setup/) and [the bevy_egui docs](https://github.com/mvlabat/bevy_egui) for instructions.

## Current Progress

To see the progress of the solver in the benchmarks look [here](./benchmark_results.md).

## Contributing

If you have ideas to improve the solver in any way, feel free to open a pull request or to create an issue. I'm happy to accept any fixes or improvements. Just make sure:

- Code is formatted with `cargo fmt`.
- No clippy warnings. (`cargo clippy`)
- All tests still pass, and new tests are added for new functionality.
