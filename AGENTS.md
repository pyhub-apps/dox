# Repository Guidelines

## Project Structure & Module Organization
- `src/cli/`: CLI entry and subcommands (`replace`, `create`, `template`, `generate`, `extract`, `config`).
- `src/core/`: Core logic — documents (`document/{word,powerpoint}.rs`), markdown, template, replace.
- `src/error/`: Unified errors, recovery, types, and tests (`src/error/tests.rs`).
- `src/utils/`: Config, UI helpers, shared utilities.
- `src/logging.rs`: Logging setup and helpers.
- `docs/`: Internal docs (e.g., `ERROR_HANDLING.md`).

## Build, Test & Development Commands
- Build: `cargo build` (debug), `cargo build --release` (optimized binary).
- Run: `cargo run -- <subcommand> [args]` (e.g., `cargo run -- replace --rules rules.yml --path ./docs`).
- Test: `cargo test` (unit/integration tests).
- Lint: `cargo clippy -- -D warnings` (treat warnings as errors).
- Format: `cargo fmt` or `cargo fmt --check` (CI-friendly check).
- Benchmarks: `cargo bench` (Criterion reports in `target/criterion`).
- Features: default enables `keyring`; disable with `--no-default-features` (or `--features no-keyring`).

## Coding Style & Naming Conventions
- Language: Rust 2021 edition; 4-space indentation.
- Names: `snake_case` for functions/vars, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Errors: prefer `thiserror` + `anyhow::Context`; return `DoxError` where applicable.
- Logging: use `tracing` with structured fields; respect `RUST_LOG`, `DOX_DEBUG`, `DOX_QUIET`.
- Keep modules small and cohesive; place command-specific logic under `src/cli/commands/`.

## Testing Guidelines
- Frameworks: `rstest`, `pretty_assertions`, `mockito` for HTTP; use async tests where needed.
- Location: colocate tests in the same module (inline `mod tests`) or under `src/*/tests.rs`.
- Naming: test functions `snake_case`, describe behavior (e.g., `replaces_multiple_matches`).
- Run locally: `cargo test`; add focused runs with `cargo test path::module::test_name`.

## Commit & Pull Request Guidelines
- Commits: follow Conventional Commits — `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:` (e.g., `feat: implement comprehensive error handling and logging`).
- PRs: include clear description, rationale, and scope; link issues (`Closes #123`); include sample command/output for CLI changes; note breaking changes.
- CI hygiene: ensure `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` pass.

## Security & Configuration Tips
- API keys: use env vars (e.g., `OPENAI_API_KEY`) or `dox config` subcommands.
- Config file: platform config dir `dox/config.toml` (Linux `~/.config/dox/`, macOS `~/Library/Application Support/dox/`, Windows `%APPDATA%\dox\`).
- Logging format: set `DOX_LOG_JSON=1` for JSON logs; tune verbosity via `RUST_LOG=debug`.
