# CHANGELOG

## Unreleased
- Skip directories nested under hidden folders (such as git worktrees in `.pi` or `.claude`) from jumps, `gt jump`, and interactive search.

## v0.4.0
- Add interactive list-based select.
- Learn directories automatically from zsh `cd` usage.
- Rank jumps using recency and frecency instead of count alone.
- Match multi-term queries against aliases and full paths.
- Skip hidden directories when indexing recursively with `gt add -a/-r` and `gt rm -a/-r`.
- `gt ls` highlights only the indexed folder, not other folders that share its name.
- `gt init` no longer leaks a `result` variable into the shell or honours the stray `_ZO_ECHO` setting.
- Publish `aarch64-apple-darwin` release binaries.
- Require Rust 1.85 (edition 2024); update dependencies and drop `itertools`.

## `v0.3.0`
- Search considers sparsity of search term-alias subsequence.

## `v0.2.4`
- Clean up error message.
- Refine tree printing.

## `v0.2.3`
- Fix `rm` bug in cli.

## `v0.2.2`
- Fix db initialisation process.

## `v0.2.1`
- Add space to tree print.
- Add demo screenshot to docs.

## `v0.2.0`
- Fix bug of clashing aliases.
- A `rm` feature to delete alises.
- Lists index in tree format.

## `v0.1.0`
- Initial implementation with simple fuzzy jump-to.
