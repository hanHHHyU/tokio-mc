# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds library code. Key modules: `client/` (async + sync clients), `codec/` (encode/decode), `frame/` (MC protocol types and parsing), `server/` (feature-gated test server), plus `lib.rs`, `error.rs`, and `header.rs`.
- `tests/` contains integration tests. Shared test helpers live in `tests/common/`.
- `examples/` contains runnable examples such as `3e-client.rs` and `mitsubishi-mc-test-server.rs`.

## Build, Test, and Development Commands
- `cargo check` to validate compilation quickly.
- `cargo build` to build the library.
- `cargo build --features "3e-async"` or `cargo build --features "3e-sync"` to build with feature flags.
- `cargo test` for the full test suite.
- `cargo test --features "3e-async"` / `cargo test --features "3e-sync"` / `cargo test --features "server"` for feature-specific coverage.
- `cargo run --example 3e-client --features "3e-async"` to run the async example.

## Coding Style & Naming Conventions
- Use standard Rust style with rustfmt (`cargo fmt`) and lint with `cargo clippy --all-features`.
- Indentation follows rustfmt (4 spaces).
- Naming: modules and files in `snake_case`, types in `CamelCase`, constants in `SCREAMING_SNAKE_CASE`.

## Testing Guidelines
- Unit tests are inline in modules under `src/`; integration tests live in `tests/`.
- Test names follow the pattern `test_<function>_<scenario>_<expected>` (see `TDD.md`).
- Use `#[tokio::test]` for async tests and follow the Arrange, Act, Assert pattern.

## Commit & Pull Request Guidelines
- Recent history uses conventional prefixes like `feat:` and `fix:`; follow that style when possible, with short, imperative summaries (optionally `closes #N`).
- PRs should describe the behavior change, list tests run, and note required feature flags. Include example updates if public behavior changes.

## Architecture & Feature Flags
- Feature gates: `3e-async`, `3e-sync`, and `server`. Use them in builds/tests to keep the API surface focused.
- The `Context` wrapper adapts address formats for Mitsubishi vs Keyence; prefer it for client operations unless you are extending lower-level protocol types.
