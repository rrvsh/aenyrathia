# AGENTS Guide for `aenyrathia`

This guide is for coding agents working in this repository.
Use it as the default source for build/test/style behavior.

## Project Snapshot
- Language: Rust (`edition = 2024`)
- App type: Axum web app + Askama templates + SQLx Postgres
- Entrypoint: `src/main.rs`
- Key modules:
  - Routes: `src/routes/`
  - App settings/state: `src/app/`
  - Git-backed wiki storage: `src/git.rs`
  - Path/content helpers: `src/formatting.rs`

## Additional Rule Files
Checked for extra agent rules:
- `.cursor/rules/`: not present
- `.cursorrules`: not present
- `.github/copilot-instructions.md`: not present

If these files are later added, merge their guidance into this doc.

## Tooling and Environment
- Build system: Cargo
- Task runner: `just` (`Justfile`)
- Optional dev shell: Nix (`flake.nix`)
- Local DB: Postgres (`docker-compose.yml`)

Dev shell tools include:
- `cargo`, `rustc`, `rustfmt`, `clippy`
- `just`
- `sqlx-cli`
- `docker-compose`, `postgresql`

## Build, Run, Lint, Format
Prefer `just` targets when available.

### Common commands
- Format: `just format` (runs `cargo fmt`)
- Lint autofix: `just lint` (runs `cargo clippy --fix --allow-dirty`)
- Format + lint: `just nice`
- Build: `just build` (runs `cargo build`)
- Run (generic): `just run`
- Run server mode: `just serve`
- Run render mode: `just render`

### Direct Cargo equivalents
- `cargo fmt`
- `cargo clippy --fix --allow-dirty`
- `cargo build`
- `cargo run -- serve`

## Test Commands
There are currently no Rust tests in `src/` or `tests/`.
Use these commands for new tests.

### Run all tests
- `cargo test`

### Run a single test (substring match)
- `cargo test <test_name_substring>`

### Run one exact unit test
- `cargo test module::tests::test_name -- --exact`

### Run one integration test target
- `cargo test --test <integration_test_file_stem>`

### Run one integration test function
- `cargo test --test <integration_test_file_stem> <test_name_substring>`

### Show output for debugging
- `cargo test <pattern> -- --nocapture`

### List discovered tests
- `cargo test -- --list`

## Database and SQLx
- Migrations live in `migrations/`
- Startup runs migrations with `sqlx::migrate!().run(&db)`
- SQLx metadata files exist in `.sqlx/`
- Start local Postgres: `just setup`
- Reset DB volume/data: `just reset`

When schema/query changes are made, keep migration and SQLx metadata workflow consistent.

## Code Style and Conventions
Clippy is strict in `Cargo.toml` and denies:
- `correctness`, `suspicious`, `complexity`, `perf`
- `style`, `pedantic`, `cargo`, `nursery`

Treat rustfmt + clippy compliance as required, not optional.

### Imports
- Group imports by origin (`std`, external crates, internal modules)
- Prefer explicit imports; avoid wildcard imports
- Let rustfmt decide wrapping/alignment
- Use trait alias imports only when needed (e.g. `use std::fmt::Write as _;`)

### Formatting
- Run `cargo fmt` after edits
- Keep formatting rustfmt-driven
- Avoid manual formatting that fights rustfmt

### Types and data modeling
- Use small explicit structs for form/query payloads (`#[derive(Deserialize)]`)
- Use explicit Askama context structs (`#[derive(Template)]`)
- Prefer concrete types over unnecessary indirection
- Keep shared runtime state in `AppState`

### Naming
- Types/traits: `PascalCase`
- Functions/variables/modules: `snake_case`
- Constants/statics: `SCREAMING_SNAKE_CASE`
- Keep existing router wrapper pattern: `AuthRouter` / `WikiRouter` + `build()`

### Error handling
- In request handlers, return `Result<_, StatusCode>` or `StatusCode`
- Map failures to HTTP status with `map_err` where possible
- Log operational/rendering failures via `log::{error,warn,trace}`
- Avoid adding `unwrap()`/`expect()` in request-path logic
- Startup/background paths may panic in existing code; keep new panics rare and justified

### Async and concurrency
- Keep Axum handlers async and non-blocking
- `git.rs` runs background sync in a dedicated thread with backoff
- Preserve sync semantics unless intentionally changing behavior

### Routing and handlers
- Add routes in route modules (`src/routes/auth.rs`, `src/routes/wiki.rs`)
- Keep query/form structs close to their handlers
- Use `Redirect` for navigation and `StatusCode` for operation results

### Templates and rendering
- Templates are in `templates/`
- Keep template context minimal and explicit
- Escape/sanitize dynamic HTML unless trust is explicit and documented

### Wiki/Git domain constraints
- Article path convention: `wiki/<slug>.md`
- Default article slug: `index`
- Default read branch: `prime`
- Edit mode branch convention: `user/<slug>`
- Preserve these conventions unless requirements explicitly change

## Agent Checklist Before Finishing
1. `cargo fmt`
2. `cargo clippy --fix --allow-dirty` (or at minimum `cargo clippy`)
3. `cargo build`
4. `cargo test` (or a targeted single-test command)
5. If DB-related: validate migrations and SQLx metadata workflow

## Operational Commands
- Start local Postgres: `just setup`
- Reset local Postgres data: `just reset`
- Build and run container: `just run-docker`
- Generate deploy SSH key + known_hosts: `just keygen`

## What to Avoid
- Skipping rustfmt/clippy for final changes
- Broad `#[allow(...)]` without tight, documented scope
- Casual changes to branch semantics (`prime`, `user/...`)
- Adding dependencies without clear need
