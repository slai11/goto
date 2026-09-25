# Repository Guidelines

## Project Structure

This repository is a Rust 2024 single-binary crate (`goto-rs`). Application code lives in `src/`: `app.rs` defines the CLI, `main.rs` dispatches commands, and the remaining modules handle initialization, database storage, indexing, directory ranking, and output formatting. Tests are `#[test]` functions alongside the code they cover; there is no separate test directory. `doc/` contains the demo image, while `.github/workflows/` holds CI and release tooling.

## Build, Test, and Development

The minimum supported Rust is 1.85 (`rust-version` in `Cargo.toml`); check compatibility with `cargo +1.85.0 test`. Common commands:

- `cargo build` — compile the binary; use `cargo build --release` for an optimized build.
- `cargo test` — run the test suite; `cargo test ranked_matches` filters by test name.
- `cargo fmt --all -- --check` — verify formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` — run lint checks and treat warnings as errors.

CI also builds and tests with `--locked`; use that flag when validating dependency-lockfile changes.

## Code and Test Conventions

Follow standard Rust formatting (four-space indentation), use `snake_case` for modules, functions, and variables, and keep command behavior in the module responsible for that concern. Add focused `#[test]` cases near the implementation, using in-memory data or temporary directories where practical. Tests must not read or write the real `~/.config/goto/db.txt`; database helpers use that path directly. Commands that resolve a jump must print only the target path to stdout; send diagnostics to stderr.

## Commits and Pull Requests

Recent commits use concise imperative subjects, sometimes with conventional prefixes such as `feat:` or `ci:`. Keep changes focused and describe the user-visible behavior or maintenance task. Pull requests should explain the change, link a related issue when applicable, and report relevant checks (`cargo test`, formatting, and Clippy). Update README or changelog documentation when CLI behavior or release-facing details change.
