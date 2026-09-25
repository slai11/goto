# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build                         # debug build (binary: target/debug/goto-rs)
cargo test                          # run all tests
cargo test ranked_matches           # run tests whose name contains the filter
cargo fmt --all -- --check          # CI formatting check
cargo clippy --all-targets --all-features -- -D warnings   # CI lint (warnings are errors)
```

Edition 2024 with MSRV 1.85 (`rust-version` in `Cargo.toml`; clap 4.6 sets that floor). CI tests 1.85 and stable across Linux (plus musl/arm via `cross`), macOS, and Windows, and runs clippy on stable. There is deliberately no `rust-toolchain.toml`: it would override the toolchain each CI matrix job installs. Check MSRV locally with `cargo +1.85.0 test`.

## Architecture

The crate is `goto-rs`, a single binary with no lib target. Users never call it directly. `goto-rs init` (`src/init.rs`) prints a zsh snippet that defines a `gt` shell function, because a child process can't change its parent shell's working directory:

- `gt <query>` runs `goto-rs <query>`. If stdout is a directory, the shell `cd`s into it. Otherwise it echoes the output, which is how `gt ls` and `gt jump` show listings.
- `gt` with no args runs `goto-rs search`, an interactive `inquire` selector, and `cd`s to the selected path.
- A `chpwd` hook calls the hidden `goto-rs record <path>` subcommand on every directory change. That is how the tool learns from plain `cd` usage.

**Stdout contract:** for commands that resolve a jump, stdout must contain only the target path. Put errors and diagnostics on stderr. `main.rs` prints errors as `[gt error]: ...` and exits 1.

### Modules
- `app.rs`: the clap `Command` definition. Top-level positional `query` args are the default action when no subcommand matches.
- `main.rs`: dispatches subcommands.
- `db.rs`: storage. The DB is a headerless CSV at `~/.config/goto/db.txt` with rows `alias,path,count,last_accessed`. It's loaded into a `HashMap<alias, GotoFile>` and fully rewritten on every change, so row order doesn't matter. `count` or `last_accessed` may be missing in older DBs; those default to 0 and `None`.
- `indexer.rs`: manual `add`, `rm`, and `prune`, plus `insert_path`. Aliases default to the directory's basename. When two paths share a basename, `get_shortest_distinct_paths` re-keys both entries to the shortest distinguishing path suffixes (e.g. `b/c` vs `a/c`). `add` and `rm` share `walk_current_dir` for the `-a`/`-r <depth>` traversal. Hidden (dot) directories are skipped during recursive indexing, but you can still add them explicitly.
- `switch.rs`: ranking. `ranked_matches` scores each query term against the alias, the full path, and path tokens (exact, prefix, substring, subsequence). Every term must match. The summed match score sorts first. `frecency_score` (visit count capped at 100, plus bucketed recency) breaks ties, followed by recency, count, and length. `ranked_paths_for_jump` backs `gt jump`: it is `ranked_matches` with no terms, filtered to `count > 0` (visited) entries. Jumping calls `db::touch_path`, which bumps the count and timestamp.
- `pretty_print.rs`: tree output for `gt ls` and numbered output for `gt jump`.

### Tests
Tests are bare `#[test]` functions at the bottom of each module, not inside `mod tests`. They exercise pure functions on in-memory `HashMap`s or on temp dirs. `db::read_db` and `db::write_db` always use the real `~/.config/goto/db.txt`, so tests must not call them or any function that does, such as `touch_path`, `learn_path`, `update`, or `switch_to_*`.
