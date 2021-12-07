[![Rust](https://github.com/GothAck/layeredconf/actions/workflows/rust.yml/badge.svg)](https://github.com/GothAck/layeredconf/actions/workflows/rust.yml) ![Crates.io](https://img.shields.io/crates/v/layeredconf) ![Crates.io](https://img.shields.io/crates/l/layeredconf) ![docs.rs](https://img.shields.io/docsrs/layeredconf)

<!-- cargo-sync-readme start -->

# LayeredConf

Yet Another Config Package

## Future

Hopefully this one will be useful to someone. Incoming features:
- More Documentation
- Features

## Features
- Generate config Layers loaded from multiple sources: files, strings, command line arguments...
- Uses Clap to auto-generate command line help + usage info
- Most of Clap's derive features are usable
- Can define futher config files to load within config files, or command line options

## Quick Example

Add `layeredconf`, `clap`, and `serde` to your `Cargo.toml`

```toml
[dependancies]
layeredconf = "0.1.3"
clap = "3.0.0-beta.5"
serde = { version = "1.0", features = ["derive"] }
```

Define your config

```rust,ignore
use layeredconf::{Builder, Format, LayeredConf, Source};
use serde::Deserialize;

#[derive(LayeredConf, Deserialize)]
struct Config {
    /// Will also load this config file
    #[layered(load_config)]
    #[clap(long)]
    config: Option<std::path::PathBuf>,

    /// Required to be set in at least one Layer (config file, command line, etc.)
    #[clap(long)]
    name: String,

    /// Optional field
    #[clap(long)]
    input: Option<String>,

    /// Defaulted field
    #[layered(default)]
    #[clap(long)]
    number: u32,
}

fn main() -> anyhow::Result<()> {
    let config: Config = Builder)::new()
        .new_layer(Source::OptionalFile {
            path: "/etc/my_app/config.yaml",
            format: Format::Auto,
        })
        .new_layer(Source::File {
            path: "relative/config.yaml",
            format: Format::Auto,
        })
        .new_layer(Source::Arguments)
        .solidify()?;

    // Use config in your application
}
```

<!-- cargo-sync-readme end -->
