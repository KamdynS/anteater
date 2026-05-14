# Anteater

Long-term goal: a debugger. Currently building `anteater-core` by translating Sy Brand's "Building a Debugger" (C++) to Rust, chapter by chapter.

## Project layout

- `crates/anteater-core/src/` — library code (process.rs, error.rs, etc.)
- `crates/anteater-core/src/bin/sdb.rs` — debugger binary
- `crates/anteater-core/tests/` — integration tests (one file per topic, e.g. `process.rs`)

Run the binary: `cargo run -p anteater-core --bin sdb -- <program-or-pid-args>`
Run tests: `cargo test -p anteater-core`

## C++ reference

The book's C++ source lives at `./sdb` (gitignored), with one git branch per chapter (`chapter-3`, `chapter-4`, `chapter-5`, ...). Always check out the relevant branch before referencing it: `cd sdb && git checkout chapter-N`.

## Working agreement

- **The user writes all the Rust.** Claude's job is to explain the C++, translate concepts, answer Rust questions, and review what the user writes. Do not write Rust unless explicitly asked ("write this for me", "I want to skip this part", etc.).
- Tests, scaffolding, and "boring" code (e.g. trivial wrappers Rust gives for free) are fair game to write when the user asks.
- When reviewing the user's Rust: point at specific lines with the issue. Don't lecture.

## Code style (this repo)

- No OOP patterns: no getters/setters, no factories, no builders, no opaque types unless there's a real reason. Fat structs with `pub` fields and methods that operate on them.
- No comments unless the *why* is non-obvious.
- Use `nix` crate for syscalls (ptrace, fork, waitpid, signals, pipe2). `thiserror` for the error enum. `rustyline` for the REPL.
- `Result<T>` aliased in `error.rs`; the `Error` enum uses `#[from]` for auto-conversion.

## Where we are

Track current chapter progress in conversation, not here — this file should stay stable.
