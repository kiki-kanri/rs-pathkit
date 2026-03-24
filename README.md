# pathkit

[![crates.io](https://img.shields.io/crates/v/pathkit)](https://crates.io/crates/pathkit)
[![docs.rs](https://docs.rs/pathkit/badge.svg)](https://docs.rs/pathkit)
[![codecov][codecov-src]][codecov-href]
[![License][license-src]][license-href]

A Rust library that provides a `Path` structure similar to Python's pathlib, with both synchronous and asynchronous file manipulation methods.

- [✨ Release Notes](./CHANGELOG.md)

## Features

- **Path Operations**: Extended path manipulation methods beyond `std::path::Path`
- **Synchronous I/O**: Blocking file system operations via `SyncFsOps` trait
- **Asynchronous I/O**: Non-blocking file system operations via `AsyncFsOps` trait (requires `async-fs-ops` feature)
- **Serde Support**: Serialize and deserialize Path with `#[derive(Serialize, Deserialize)]`
- **Path Joining**: Use `/` operator for intuitive path composition

## Installation

Add to your `Cargo.toml`:

```bash
cargo add pathkit
```

For async support:

```bash
cargo add pathkit --features async-fs-ops
```

## Usage

### Basic Path Operations

```rust
use pathkit::Path;

// Create a new path
let path = Path::new("/home/user/project");

// Join paths
let config = path.join("config.json");
let nested = path / "subdir" / "file.txt";  // Using / operator

// Path components
let parent = path.parent();
let file_name = path.file_name();
let extension = path.extension();
```

### Synchronous File Operations

```rust
use pathkit::{Path, SyncFsOps};

let path = Path::new("/tmp/test.txt");

// Read/write files
path.write_sync(b"Hello, world!")?;
let content = path.read_sync()?;

// Check existence and type
if path.exists_sync()? {
    println!("File size: {}", path.get_file_size_sync()?);
}

// Create directories
Path::new("/tmp/new_project").create_dir_all_sync()?;

// Read JSON
#[derive(Deserialize)]
struct Config { name: String }
let config: Config = path.read_json_sync()?;
```

### Asynchronous File Operations (with `async-fs-ops` feature)

```rust
use pathkit::{Path, AsyncFsOps};

let path = Path::new("/tmp/test.txt");

// Async read/write
path.write(b"Hello, world!").await?;
let content = path.read().await?;

// Async directory operations
Path::new("/tmp/new_project").create_dir_all().await?;
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `async-fs-ops` | Enable async file system operations (requires tokio) |
| `full` | Enable all features |

## Platform Support

- **Unix/Linux/macOS**: Full support including `chmod`, `chown`, and special file type checks
- **Windows**: Core functionality supported; some Unix-specific features are conditionally compiled out

## API Overview

### Core Path Methods
- `new()`, `join()`, `parent()`, `with_extension()`
- `absolutize()`, `absolutize_from()`, `absolutize_virtually()`
- `canonicalize()`
- `is_absolute()`, `is_relative()`

### File System Operations (SyncFsOps)
- `exists_sync()`, `is_file_sync()`, `is_dir_sync()`, `is_symlink_sync()`
- `read_sync()`, `write_sync()`, `read_to_string_sync()`
- `read_json_sync()`, `write_json_sync()`
- `create_dir_sync()`, `create_dir_all_sync()`, `remove_dir_sync()`
- `remove_file_sync()`, `remove_dir_all_sync()`
- `metadata_sync()`, `get_file_size_sync()`, `truncate_sync()`
- `set_permissions_sync()`, `read_dir_sync()`, `read_dir_names_sync()`, `read_dir_paths_sync()`, `empty_dir_sync()`
- `chmod_sync()`, `chown_sync()` (Unix only)
- `is_block_device_sync()`, `is_char_device_sync()`, `is_fifo_sync()`, `is_socket_sync()` (Unix only)
- `copy_file_sync()`, `hard_link_sync()`, `soft_link_sync()` (Unix only)
- `read_link_sync()`, `symlink_metadata_sync()` (Unix only)
- `touch_sync()`

### Trait Implementations
`Path` implements a rich set of standard library traits for interoperability:
- `AsRef<Path>` / `AsRef<PathBuf>` / `AsRef<OsStr>` / `AsRef<str>` — use `Path` wherever these types are expected
- `Borrow<Path>` — use `Path` as a map key with `std::collections` hash types
- `Deref<Target = Path>` — transparent access to `std::path::Path` methods
- `Display` / `Debug` / `Serialize` / `Deserialize` — string-like and serde support
- `From<&str>` / `From<&Path>` / `From<String>` / `From<Path> for String` — seamless conversions
- `Div<&str>` / `Div<String>` / `Div<&Path>` / `Div<Path>` — use `/` operator: `path / "subdir"`

### File System Operations (AsyncFsOps)
Same operations as SyncFsOps but async:
- `exists()`, `is_file()`, `is_dir()`, `is_symlink()`
- `read()`, `write()`, `read_to_string()`
- `read_json()`, `write_json()`
- `create_dir()`, `create_dir_all()`, `remove_dir()`
- `remove_file()`, `remove_dir_all()`
- `metadata()`, `get_file_size()`, `truncate()`
- `set_permissions()`, `read_dir()`, `empty_dir()`
- `chmod()`, `chown()` (Unix only)
- `is_block_device()`, `is_char_device()`, `is_fifo()`, `is_socket()` (Unix only)

## License

[MIT License](./LICENSE)

<!-- Badges -->
[codecov-href]: https://codecov.io/gh/kiki-kanri/rs-pathkit
[codecov-src]: https://codecov.io/gh/kiki-kanri/rs-pathkit/graph/badge.svg?token=qEBTmimZmx

[license-href]: https://github.com/kiki-kanri/rs-pathkit/blob/main/LICENSE
[license-src]: https://img.shields.io/github/license/kiki-kanri/rs-pathkit?colorA=18181b&colorB=28cf8d&style=flat
