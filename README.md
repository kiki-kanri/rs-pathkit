# pathkit

[![crates.io](https://img.shields.io/crates/v/pathkit)](https://crates.io/crates/pathkit)
[![docs.rs](https://docs.rs/pathkit/badge.svg)](https://docs.rs/pathkit)
[![codecov][codecov-src]][codecov-href]
[![License][license-src]][license-href]

A Rust library that provides a `Path` structure similar to Python's pathlib, with both synchronous and asynchronous file manipulation methods.

- [✨ Release Notes](./CHANGELOG.md)

## Features

- 🧭 **Path Operations**: pathlib-style `Path` wrapper around `std::path::PathBuf`
- 🧩 **Path Macro**: `path!` supports direct path expressions and `format!`-style construction
- ➗ **Path Joining**: use `/` for concise path composition
- 📁 **Synchronous I/O**: blocking file system operations via `SyncFsOps`
- ⚡ **Asynchronous I/O**: non-blocking file system operations via `AsyncFsOps` (requires `async-fs-ops`)
- 🔄 **Serde Support**: serialize and deserialize `Path`

## Installation

Add to your `Cargo.toml`:

```bash
cargo add pathkit
```

Optional features:

```bash
cargo add pathkit --features async-fs-ops
cargo add pathkit --features full
```

## Usage

### Basic Path Operations

```rust
use pathkit::path;

let root = path!("/home/{}/project", "user");
let config = &root / "config" / "app.json";

// `join` is still available for std-like APIs.
let readme = root.join("README.md");

let parent = config.parent();
let file_name = config.file_name();
let extension = config.extension();
```

### Synchronous File Operations

```rust
use pathkit::{
    SyncFsOps,
    path,
};

let directory = tempfile::tempdir()?;
let file = path!(directory.path()) / "message.txt";

file.write_sync(b"Hello, world!")?;
let content = file.read_sync()?;
let size = file.get_file_size_sync()?;

let moved = file.rename_to_sync(path!(directory.path()) / "moved.txt")?;
let canonical = moved.canonicalize_sync()?;
```

Use `open_with_options_sync()` when you need custom `std::fs::OpenOptions`.
Use `write_atomic_sync()` to replace a file without readers observing partial contents.

### Asynchronous File Operations

Requires the `async-fs-ops` feature.

```rust
use pathkit::{
    AsyncFsOps,
    path,
};

let directory = tempfile::tempdir()?;
let file = path!(directory.path()) / "nested" / "message.txt";

file.create_parent_dir_all().await?;
file.write(b"Hello, world!").await?;
let content = file.read().await?;

let moved = file.rename_to(path!(directory.path()) / "moved.txt").await?;
let canonical = moved.canonicalize().await?;
```

Use `open_with_options()` when you need custom `tokio::fs::OpenOptions`.
Use `write_atomic()` to replace a file without readers observing partial contents.

### JSON Files

JSON methods return `serde_json::Error`; filesystem failures are wrapped and can be
inspected with `is_io()` and `io_error_kind()`.

```rust
use pathkit::{
    SyncFsOps,
    path,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Config {
    name: String,
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let file = path!(directory.path()) / "config.json";
    let config = Config {
        name: "app".to_owned(),
        port: 8080,
    };

    file.write_json_sync(&config)?;
    let loaded: Config = file.read_json_sync()?;
    assert_eq!(loaded, config);
    Ok(())
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `async-fs-ops` | Enable async file system operations via tokio |
| `all` | Enable all optional features |
| `full` | Alias of `all` |

## Platform Support

- **Unix/Linux/macOS**: full support including `chmod`, `chown`, and special file type checks
- **Windows**: core functionality supported; Unix-specific APIs are conditionally compiled out

## API Overview

Main entry points:

- `Path` — owned path wrapper around `std::path::PathBuf`
- `path!` — convenient path construction macro
- `/` operator — concise path composition
- `SyncFsOps` — blocking filesystem operations
- `AsyncFsOps` — async filesystem operations with tokio
- `PathEntry` / `AsyncPathEntry` — typed directory entries

See [docs.rs](https://docs.rs/pathkit) for the complete method list.

## License

[MIT License](./LICENSE)

<!-- Badges -->
[codecov-href]: https://codecov.io/gh/kiki-kanri/rs-pathkit
[codecov-src]: https://codecov.io/gh/kiki-kanri/rs-pathkit/graph/badge.svg?token=qEBTmimZmx

[license-href]: https://github.com/kiki-kanri/rs-pathkit/blob/main/LICENSE
[license-src]: https://img.shields.io/github/license/kiki-kanri/rs-pathkit?colorA=18181b&colorB=28cf8d&style=flat
