# {{crate}}

[![crates.io](https://img.shields.io/crates/v/{{crate}})](https://crates.io/crates/{{crate}})
[![docs.rs](https://docs.rs/{{crate}}/badge.svg)](https://docs.rs/{{crate}})
[![MSRV](https://img.shields.io/crates/msrv/{{crate}})](./Cargo.toml)
[![codecov][codecov-src]][codecov-href]
[![License][license-src]][license-href]

{{One sentence: what the crate provides and its main differentiator.}}

- [✨ Release Notes](./CHANGELOG.md)

## Installation

```bash
cargo add {{crate}}
```

<!-- Drop this section if the crate has no optional features. -->
## Features

No features are enabled by default.

| Feature | Enables |
| --- | --- |
| `full` | Every feature below |
| `{{feature}}` | {{One line: what it adds}} |

<!-- Optional: only when behavior differs by platform or a platform is unsupported. -->
## Platform Support

{{e.g. Unix and Windows; `{{feature}}` is Unix only.}}

## License

[MIT License](./LICENSE)

<!-- Badges -->
[codecov-href]: https://codecov.io/gh/{{owner}}/{{repo}}
[codecov-src]: https://codecov.io/gh/{{owner}}/{{repo}}/graph/badge.svg?token={{codecov_token}}

[license-href]: https://github.com/{{owner}}/{{repo}}/blob/main/LICENSE
[license-src]: https://img.shields.io/github/license/{{owner}}/{{repo}}?colorA=18181b&colorB=28cf8d&style=flat
