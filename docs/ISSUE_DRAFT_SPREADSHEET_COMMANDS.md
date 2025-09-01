# Feat: Add `dox excel` and `dox gsheet` subcommands via multi-crate architecture

## Summary
Introduce spreadsheet-specific subcommands to the `dox` CLI: `dox excel` for local XLSX and `dox gsheet` for Google Sheets. Implement a modular architecture using a Cargo workspace with provider crates to keep dependencies lean and features optional.

## Goals
- Provide first-class subcommands: `dox excel` and `dox gsheet`.
- Extract shared logic (errors, config, logging, rules) into `dox-core`.
- Isolate providers into `dox-excel` and `dox-gsheet` crates.
- Gate subcommands behind features (`excel`, `gsheet`) for slim binaries.
- Ensure solid tests, logging, error handling, and retry policies.

## Non-Goals
- Full rewrite of existing document features; focus on spreadsheet providers.
- GUI or web interface.

## Architecture
- Cargo workspace with members: `dox-core` (lib), `dox-excel` (lib), `dox-gsheet` (lib), `dox-cli` (bin).
- Trait: `SpreadsheetProvider` (async) in `dox-core` with `read_range`, `write_range`, `list_sheets`, `apply_rules`.
- Common types: `SheetId`, `RangeRef` (A1), `Cell`, `Ruleset`.
- CLI (`dox-cli`) conditionally compiles `excel`/`gsheet` subcommands via `#[cfg(feature = "excel")]` / `#[cfg(feature = "gsheet")]`.

## CLI Surface (initial)
- `dox excel replace --rules rules.yml --path data.xlsx`
- `dox excel extract --path data.xlsx --range Sheet1!A1:D100 --out out.csv`
- `dox gsheet replace --rules rules.yml --sheet <SPREADSHEET_ID>`
- `dox gsheet sync --sheet <ID> --dir ./local --mode {pull,push}`

## Dependencies
- `dox-excel`: `calamine` (read), `rust_xlsxwriter` (write).
- `dox-gsheet`: `reqwest`, `yup-oauth2`, Google Sheets v4 API client (or REST), `secrecy/keyring`.

## Config & Security
- Extend `Config` with `excel` and `gsheet` sections (formats, credentials path, token store, scopes, batch size).
- ENV: `GOOGLE_APPLICATION_CREDENTIALS`, `DOX_GSHEET_TOKEN_PATH`.

## Testing
- `dox-excel`: fixtures + temp dirs; streaming for large files.
- `dox-gsheet`: `mockito` for API; rate limit and retry scenarios.
- CLI integration: `cargo test -p dox-cli --features excel,gsheet`.

## Acceptance Criteria
- New subcommands appear under corresponding features.
- Read/write operations validated by tests.
- Error handling integrates with `DoxError` and logging.
- Docs updated (README + usage examples).

## Tasks
- [ ] Create workspace; extract `dox-core`.
- [ ] Implement `SpreadsheetProvider` abstraction.
- [ ] Add `dox-excel` with basic ops and CLI wiring.
- [ ] Add `dox-gsheet` with auth, basic ops and CLI wiring.
- [ ] Extend config and env handling; docs.
- [ ] Tests: unit + integration; CI matrix for features.
