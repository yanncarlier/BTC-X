# AGENTS.md - Agentic Coding Guidelines for Rust Projects

## Overview
This document provides guidelines for AI coding agents working in this Rust codebase. Adjust the specifics (crate names, paths, workspace layout) to match the actual project before use.

## Build/Lint/Test Commands

### Building
```bash
# Build all workspace members in debug mode
cargo build --workspace

# Build all workspace members in release mode
cargo build --workspace --release

# Build a single package
cargo build --package <package_name>

# Check without producing binaries (fast feedback loop)
cargo check --workspace
```

### Testing
```bash
# Run all tests in the workspace
cargo test --workspace

# Run a specific test by name
cargo test test_function_name

# Run tests for a specific package
cargo test --package <package_name>

# Run tests with output printed (useful for debugging)
cargo test --workspace -- --nocapture

# Run only doc tests
cargo test --doc

# Run a single ignored/expensive test explicitly
cargo test test_function_name -- --ignored
```

### Linting and Formatting
```bash
# Format code (run before committing)
cargo fmt --all

# Check formatting without modifying files (use in CI)
cargo fmt --all -- --check

# Run clippy, denying warnings
cargo clippy --workspace --all-targets -- -D warnings

# Run clippy including pedantic lints (optional, stricter)
cargo clippy --workspace --all-targets -- -W clippy::pedantic
```

### Security & Dependency Hygiene
```bash
# Check for known vulnerabilities in dependencies
cargo audit

# Check licenses, duplicate deps, and banned crates (requires deny.toml)
cargo deny check

# Check for outdated dependencies
cargo outdated
```

### Cleaning
```bash
cargo clean
```

## Code Style Guidelines

### Imports and Dependencies
- Group imports: standard library, then external crates, then local/workspace crates, each group separated by a blank line.
- Prefer explicit imports over glob imports (`use std::collections::HashMap`, not `use std::collections::*`), except in test modules where `use super::*;` is idiomatic.
- Example:
```rust
use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::model::Record;
```

### Naming Conventions
- **Functions/methods**: `snake_case` (e.g., `parse_config`, `run_pipeline`)
- **Variables**: `snake_case`, descriptive (avoid single-letter names outside tight loops/closures)
- **Types (structs/enums/traits)**: `PascalCase` (e.g., `RequestHandler`, `ConnectionState`)
- **Constants/statics**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT_MS`)
- **Modules**: `snake_case` (e.g., `request_handler`, `config_loader`)
- **Generic type parameters**: single uppercase letter (`T`, `E`) or short PascalCase for meaning (`K`, `V`, `Item`)

### Error Handling
- Use `Result<T, E>` for any operation that can fail; avoid `panic!`, `unwrap()`, and `expect()` outside tests, examples, and truly unreachable states.
- Define custom error types with `thiserror` for libraries; use `anyhow` for application-level error aggregation where the caller doesn't need to match on variants.
- Propagate errors with `?` rather than manual `match`/`unwrap` chains.
- Error messages should be specific and actionable, not generic ("failed to parse config at line 12: missing `port` field", not "parse error").
- Example:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### Unsafe Code
- Default to `#![forbid(unsafe_code)]` at the crate root unless the crate has a specific, documented need (FFI, performance-critical low-level work).
- Any `unsafe` block must have a `// SAFETY:` comment directly above it explaining why the invariants hold.
- Keep `unsafe` blocks as small as possible; wrap them in safe abstractions at the module boundary.

### Code Structure and Patterns
- Prefer early returns / the `?` operator over deep nesting.
- Keep functions small and single-purpose; extract helper functions once a function handles more than one clear responsibility.
- Structure code so pure logic (no I/O, no side effects) is separated from I/O and orchestration code, to make testing easier.
- Example:
```rust
fn process_batch(records: Vec<Record>) -> Result<Summary, ProcessError> {
    if records.is_empty() {
        return Err(ProcessError::EmptyInput);
    }

    let validated = validate_records(&records)?;
    let summary = summarize(&validated);

    Ok(summary)
}
```

### Serialization and Deserialization
- Use `serde` with `#[derive(Serialize, Deserialize)]` for data-transfer structures.
- Use descriptive, stable field names for any externally-facing format (JSON/YAML/TOML); use `#[serde(rename = "...")]` if the external name differs from the idiomatic Rust name.
- Use `Option<T>` for genuinely optional fields; consider `#[serde(default)]` for fields that should fall back to a default rather than fail deserialization.
- Example:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPayload {
    pub id: String,
    pub items: Vec<Item>,
    #[serde(default)]
    pub priority: Option<u8>,
}
```

### Async Code (if applicable)
- Be explicit about the async runtime in use (e.g., `tokio`) and don't mix runtimes within a crate.
- Avoid blocking calls inside async functions; use the runtime's blocking-task spawning mechanism (e.g., `tokio::task::spawn_blocking`) for CPU-heavy or blocking I/O work.
- Prefer bounded channels (`tokio::sync::mpsc::channel` with a capacity) over unbounded ones unless there's a specific reason.

### Testing Guidelines
- Write unit tests for all public functions and any non-trivial private logic.
- Use descriptive test names that state the scenario and expectation (e.g., `parses_valid_config_with_all_fields`, `rejects_empty_input`).
- Follow Arrange/Act/Assert structure within tests.
- Test both success paths and failure/edge cases (empty input, boundary values, malformed data).
- Use `#[cfg(test)]` modules within the same file as the code under test; use `tests/` directory for integration tests that exercise the public API as a consumer would.
- Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_valid_batch_successfully() {
        let records = vec![test_record()];

        let result = process_batch(records);

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_empty_batch() {
        let result = process_batch(vec![]);

        assert!(matches!(result, Err(ProcessError::EmptyInput)));
    }

    fn test_record() -> Record {
        Record { id: "test-1".into(), value: 42 }
    }
}
```

### Documentation
- Add doc comments (`///`) for all public items: functions, structs, enums, traits, and modules.
- Include `# Arguments`, `# Returns`, and `# Errors` sections for non-trivial public functions.
- Use `# Examples` with doctested code blocks where it clarifies usage; keep examples compiling (`cargo test --doc` should pass).
- Example:
```rust
/// Parses a configuration file into a validated `Config`.
///
/// # Arguments
/// * `path` - Path to the configuration file on disk.
///
/// # Returns
/// The parsed and validated `Config` on success.
///
/// # Errors
/// Returns `ConfigError::Io` if the file cannot be read, or
/// `ConfigError::MissingField` if a required field is absent.
///
/// # Examples
/// ```
/// let config = load_config("config.toml")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    // Implementation...
}
```

### File Organization
- Use `lib.rs` for the crate's public API surface; keep it as a thin re-export layer over internal modules where possible.
- Use `main.rs` only for binary entry points: argument parsing, wiring, and calling into library code.
- One logical concern per module; avoid large "misc" or "utils" modules that accumulate unrelated code.
- Keep tests colocated with the code they test unless they're integration tests, which belong in `tests/`.

### Performance Considerations
- Choose data structures deliberately (`HashMap` for lookups, `Vec` for ordered/sequential access, `BTreeMap` when ordering matters).
- Avoid unnecessary `.clone()` calls, especially in hot paths; prefer borrowing (`&T`) where ownership isn't required.
- Prefer iterator chains over manual index-based loops where they're equally clear.
- Profile before optimizing (`cargo flamegraph`, `criterion` for benchmarks) rather than guessing at bottlenecks.

### Dependency and Version Management
- Pin major versions for critical or security-sensitive dependencies in `Cargo.toml`.
- Keep the Rust edition and MSRV (minimum supported Rust version) consistent across all workspace members; state the MSRV explicitly (e.g., in `rust-toolchain.toml` or the README).
- Justify any new dependency added to the workspace, especially ones pulling in significant transitive dependency trees or `unsafe` code.
- Re-run `cargo audit` after any dependency update.

## Common Patterns

### CLI Tool Structure
```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "Description of what this tool does")]
struct Cli {
    /// Input file path
    input: String,

    /// Optional output path
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = process(&cli.input)?;

    match cli.output {
        Some(path) => std::fs::write(path, result)?,
        None => println!("{result}"),
    }

    Ok(())
}
```

### API Handler Structure (example using `axum`)
```rust
use axum::{extract::State, http::StatusCode, Json};

async fn handler(
    State(state): State<AppState>,
    Json(req): Json<Request>,
) -> Result<Json<Response>, (StatusCode, String)> {
    if req.items.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "items required".into()));
    }

    process_request(&state, &req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
```

## Final Checklist Before Committing
1. `cargo test --workspace` passes.
2. `cargo clippy --workspace --all-targets -- -D warnings` is clean.
3. `cargo fmt --all -- --check` passes (or run `cargo fmt --all` to fix).
4. `cargo build --workspace --release` succeeds.
5. `cargo audit` (or `cargo deny check`) shows no new issues.
6. No secrets, credentials, or sensitive data committed.
7. Public API docs updated if signatures or behavior changed.

## Getting Help
- Run `cargo --list` to see all available cargo subcommands.
- Check individual `Cargo.toml` files for per-crate dependency and feature information.
- Refer to the project's `README.md` for architecture and usage context.
