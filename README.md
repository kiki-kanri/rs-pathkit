# pathkit

[![crates.io](https://img.shields.io/crates/v/pathkit)](https://crates.io/crates/pathkit)
[![docs.rs](https://docs.rs/pathkit/badge.svg)](https://docs.rs/pathkit)
[![MSRV](https://img.shields.io/crates/msrv/pathkit)](./Cargo.toml)
[![codecov][codecov-src]][codecov-href]
[![License][license-src]][license-href]

A pathlib-style `Path` for Rust with sync and optional async (Tokio) filesystem operations.

- [✨ Release Notes](./CHANGELOG.md)

## Installation

```bash
cargo add pathkit
```

## Features

No features are enabled by default.

| Feature | Enables |
| --- | --- |
| `full` | Every feature below |
| `async-fs-ops` | The `async_fs_ops` module: Tokio-backed async counterparts of the sync operations; needs a Tokio runtime |

## License

[MIT License](./LICENSE)

<!-- Badges -->
[codecov-href]: https://codecov.io/gh/kiki-kanri/rs-pathkit
[codecov-src]: https://codecov.io/gh/kiki-kanri/rs-pathkit/graph/badge.svg?token=qEBTmimZmx

[license-href]: https://github.com/kiki-kanri/rs-pathkit/blob/main/LICENSE
[license-src]: https://img.shields.io/github/license/kiki-kanri/rs-pathkit?colorA=18181b&colorB=28cf8d&style=flat
