# pathkit

[![crates.io](https://img.shields.io/crates/v/pathkit)](https://crates.io/crates/pathkit)
[![docs.rs](https://docs.rs/pathkit/badge.svg)](https://docs.rs/pathkit)
[![codecov][codecov-src]][codecov-href]
[![License][license-src]][license-href]

A pathlib-style `Path` for Rust with sync and optional async (Tokio) filesystem operations.

- [✨ Release Notes](./CHANGELOG.md)

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

## License

[MIT License](./LICENSE)

<!-- Badges -->
[codecov-href]: https://codecov.io/gh/kiki-kanri/rs-pathkit
[codecov-src]: https://codecov.io/gh/kiki-kanri/rs-pathkit/graph/badge.svg?token=qEBTmimZmx

[license-href]: https://github.com/kiki-kanri/rs-pathkit/blob/main/LICENSE
[license-src]: https://img.shields.io/github/license/kiki-kanri/rs-pathkit?colorA=18181b&colorB=28cf8d&style=flat
