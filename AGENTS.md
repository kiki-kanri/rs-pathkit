# OpenCode Agent Instructions

## Project Overview

Single-crate Rust library (`pathkit`) providing a Python pathlib-like `Path` type with sync/async file operations.

## Essential Commands

Always use these cargo aliases (defined in `.cargo/config.toml`):

```bash
# Linting (REQUIRED before commit)
cargo lint          # clippy --all-features --all-targets -- -D warnings
cargo lint-fix      # clippy --all-features --all-targets --fix

# Testing with coverage (requires cargo-llvm-cov)
cargo coverage              # Show coverage report
cargo coverage-generate     # Generate lcov.info for codecov

# Formatting (must use nightly)
cargo +nightly fmt --all
```

## Feature Flags

| Feature | Module | Description |
|---------|--------|-------------|
| `async-fs-ops` | `async_fs_ops.rs` | Async file operations (requires tokio) |
| `sea-orm` | `sea_orm.rs` | SeaORM integration for database models |
| `all` | - | Enables both features above |
| `full` | - | Alias for `all` |

Always use `--all-features` for testing, linting, and coverage.

## Development Workflow

1. **Before committing:**
   ```bash
   cargo +nightly fmt --all
   cargo lint
   cargo t --all-features
   ```

2. **Running tests:**
   - Unit tests are inline in `src/*.rs` files
   - `tests/` directory exists but is empty (reserved for integration tests)
   - Use `cargo t --all-features` to test all feature combinations

3. **VSCode settings** (see `.vscode/settings.json`):
   - rust-analyzer configured with `"all"` features
   - Format on save enabled
   - rustfmt uses `+nightly`

## Code Style

- **Formatter**: nightly rustfmt with custom config (`rustfmt.toml`)
  - `group_imports = "StdExternalCrate"`
  - `imports_granularity = "Crate"`
  - `max_width = 120`
- **Edition**: Rust 2021
- **Line endings**: LF (Unix)

## Architecture Notes

- `src/lib.rs`: Crate root, re-exports `Path`, `SyncFsOps`, and optionally `AsyncFsOps`
- `src/core.rs`: Main `Path` struct and core path operations
- `src/sync_fs_ops.rs`: `SyncFsOps` trait with blocking file operations
- `src/async_fs_ops.rs`: `AsyncFsOps` trait (feature-gated)
- `src/traits.rs`: Standard library trait implementations (`AsRef`, `Deref`, etc.)
- `src/div.rs`: `/` operator implementations for path joining
- `src/sea_orm.rs`: SeaORM integration (feature-gated)

## Release Process

See `release.sh` for automated release workflow. Key points:
- Requires clean git state
- Runs fmt check, lint, test, build in sequence
- Uses `changelogen` for changelog generation
- Publishes to crates.io

## CI/Testing

- GitHub Actions workflow: `.github/workflows/release-test-codecov.yaml`
- Tests run on Ubuntu and Windows
- Coverage uploaded to codecov
- Uses `cargo coverage` and `cargo coverage-generate` commands
