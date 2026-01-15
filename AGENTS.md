# AGENTS.md
# Repository: author
# Scope: root and all subdirectories
# Purpose: instructions for automated coding agents

## Project Overview
- This is a small Rust CLI for managing an `author.txt` file at repo root.
- `author.txt` stores whitespace-separated `firstname.lastname` logins.
- The CLI supports:
  - No args: list authors found in repo root (walk up to `.git`).
  - `add LOGINS`: add logins, de-duplicate and sort.
  - `remove`: prompt using `inquire` to select logins to remove.
  - `remove LOGINS`: remove specified logins.
- Listing outputs one login per line.
- Empty list output is italic with the `add` command in bold+italic.

## Build / Test / Lint
- Use `cargo build` for local builds.
- If needed, use `cargo build --release` for production binaries.
- There are no unit tests yet; add tests before running `cargo test`.
- There is no lint setup (no `clippy` config), but `cargo clippy` is fine if you add it.
- There is no formatting config; `cargo fmt` is acceptable after edits.

### Single Test Guidance
- There are currently no tests; do not invent a test name.
- If tests are added later, prefer: `cargo test <test_name>`.
- For a single integration test file (future), use `cargo test --test <file_stem>`.

## Repo Discovery Behavior
- All commands resolve repo root by walking upwards until `.git` exists.
- If `.git` is not found, return an error instead of writing to disk.
- `author.txt` is stored at repo root, not in `src/`.

## File Format Rules
- `author.txt` must contain whitespace-separated logins.
- Deduplicate and sort logins before writing.
- Preserve output order by using a sorted set (BTreeSet).
- When rewriting, output a single line with space-separated logins and a trailing newline.

## Code Style
### Imports
- Do not overuse `use` for deep module paths; prefer local paths when clearer.
- Import traits with `as _` to avoid name pollution (example: `use std::io::Write as _;`).
- Keep import blocks minimal and grouped by std/external/local.

### Formatting
- Follow `rustfmt` defaults; run `cargo fmt` when needed.
- Do not bother with manual formatting; run `cargo fmt` when done with edits.

### Naming
- Use descriptive, snake_case function and variable names.
- Avoid single-letter variable names unless used in short iterators.
- Constants should be `SCREAMING_SNAKE_CASE`.

### Types
- Favor explicit types when inference is ambiguous or impacts readability.
- Use `BTreeSet<String>` for sorted, deduplicated login collections.
- Prefer `&Path` for path parameters; avoid `String` where `Path` is clearer.

### Error Handling
- Use `Result<_, Box<dyn std::error::Error>>` for top-level functions.
- Prefer early returns for invalid input (e.g., missing logins on `add`).
- Avoid panics in user-facing flows; return an error instead.

### CLI Behavior
- `add` requires at least one login or returns an error.
- `remove` without args prompts with `inquire::MultiSelect`.
- `remove` with args removes those logins without prompting.
- Listing with no authors prints italic message and includes bold+italic command.

## Dependencies
- `inquire` is used for interactive prompts.
- `colored` is used for bold/italic styling in output.

## Cursor / Copilot Rules
- No `.cursor/rules`, `.cursorrules`, or `.github/copilot-instructions.md` found.

## Implementation Notes
- Avoid writing in other directories (keep all data in repo root).
- Do not introduce new config files unless explicitly requested.
- Use the smallest set of changes to satisfy requirements.
- Avoid adding inline comments unless asked.
- Keep command outputs deterministic (sorted authors).

## Suggested Future Enhancements (Do Not Implement Without Request)
- Validate login format (`firstname.lastname`).
- Add tests for add/remove/list flows.
- Support custom author file path via env or flag.
- Provide `--help` usage output.

## Example Manual Flows
- List: run binary with no args to print authors.
- Add: `author add jane.doe john.smith`.
- Remove interactive: `author remove`.
- Remove direct: `author remove jane.doe`.

## Notes on Documentation
- Prefer describing commands specific to this repo over generic Rust docs.
- Keep docs short and practical; focus on current behavior.
