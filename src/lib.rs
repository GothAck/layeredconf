#![deny(missing_docs)]

//! # LayeredConf
//!
//! Yet Another Config Package
//!
//! ## Future
//!
//! Hopefully this one will be useful to someone. Incoming features:
//! - More Documentation
//! - Features
//!
//! ## Features
//! - Generate config Layers loaded from multiple sources: files, strings, command line arguments...
//! - Uses Clap to auto-generate command line help + usage info
//! - Most of Clap's derive features are usable
//! - Can define futher config files to load within config files, or command line options
//!
//! ## Quick Example
//!
//! Add `layeredconf`, `clap`, and `serde` to your `Cargo.toml`
//!
//! ```toml
//! [dependancies]
//! layeredconf = "0.1.4"
//! clap = "3.0.0-beta.5"
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! Define your config
//!
//! ```rust,ignore
//! use layeredconf::{Builder, Format, LayeredConf, Source};
//! use serde::Deserialize;
//!
//! #[derive(LayeredConf, Deserialize)]
//! struct Config {
//!     /// Will also load this config file
//!     #[layered(load_config)]
//!     #[clap(long)]
//!     config: Option<std::path::PathBuf>,
//!
//!     /// Required to be set in at least one Layer (config file, command line, etc.)
//!     #[clap(long)]
//!     name: String,
//!
//!     /// Optional field
//!     #[clap(long)]
//!     input: Option<String>,
//!
//!     /// Defaulted field
//!     #[layered(default)]
//!     #[clap(long)]
//!     number: u32,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let config: Config = Builder)::new()
//!         .new_layer(Source::OptionalFile {
//!             path: "/etc/my_app/config.yaml",
//!             format: Format::Auto,
//!         })
//!         .new_layer(Source::File {
//!             path: "relative/config.yaml",
//!             format: Format::Auto,
//!         })
//!         .new_layer(Source::Arguments)
//!         .solidify()?;
//!
//!     // Use config in your application
//! }
//! ```

mod layers;

use std::path::{Path, PathBuf};

use thiserror::Error as ThisError;

pub use layers::{Builder, Format, Source};

/// LayeredConf Derive Macro
///
/// ```rust
/// use std::path::PathBuf;
///
/// use layeredconf::LayeredConf;
///
/// #[derive(LayeredConf, serde::Deserialize)]
/// struct Config {
///     #[layered(load_config)]
///     #[clap(long)]
///     config: Option<PathBuf>,
///     #[clap(long)]
///     name: String,
///     #[layered(subconfig)]
///     subconfig: SubConfig,
/// }
///
/// #[derive(LayeredConf, serde::Deserialize)]
/// #[layered(subconfig)]
/// struct SubConfig {
///     #[clap(long)]
///     sub_name: String,
/// }
/// ```
pub use layeredconf_derive::LayeredConf;

/// LayeredConf Result
pub type Result<T> = std::result::Result<T, Error>;

/// LayeredConf Error
#[derive(ThisError, Debug)]
pub enum Error {
    /// Solidify failed with missing fields set
    #[error("Solidify failed, missing fields {missing:?}")]
    SolidifyFailedMissing {
        /// The missing fields
        missing: Vec<String>,
    },
    /// Solidify failed, no layers
    #[error("Solidify failed, no layers")]
    SolidifyFailedNoLayers,
    /// Unknown file extension
    #[error("Unknown extension {extension:?}")]
    UnknownExtension {
        /// The file extension that failed
        extension: Option<String>,
    },
    /// File auto format detection failed
    #[error("Auto format detection failed")]
    AutoFormatFailed,
    /// A file was loaded in two Layers
    #[error("Loop detected loading config files")]
    LoopingLoadConfig,
    /// Path canonicalization failed
    #[error("Path canonicalization error {wrapped:?} for {path:?}")]
    PathCanonicalization {
        /// Wrapped io::Error
        wrapped: std::io::Error,
        /// Path that failed
        path: PathBuf,
    },
    /// File not found
    #[error("File not found {path:?}")]
    FileNotFound {
        /// Path that failed
        path: PathBuf,
    },
    /// File I/O error
    #[error("I/O Error {wrapped:?} for {path:?}")]
    IoError {
        /// Wrapped io::Error
        wrapped: std::io::Error,
        /// Path that failed
        path: PathBuf,
    },
    /// Json error
    #[error("Json Error {wrapped:?}")]
    JsonError {
        /// Wrapped error
        wrapped: serde_json::Error,
    },
    /// Toml error
    #[error("Toml Error {wrapped:?}")]
    TomlError {
        /// Wrapped error
        wrapped: toml::de::Error,
    },
    /// Yaml error
    #[error("Yaml Error {wrapped:?}")]
    YamlError {
        /// Wrapped error
        wrapped: serde_yaml::Error,
    },
}

pub(crate) fn map_io_error(path: &'_ Path) -> impl Fn(std::io::Error) -> Error + '_ {
    move |wrapped| {
        let path = path.to_path_buf();
        match wrapped.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound { path },
            _ => Error::IoError { wrapped, path },
        }
    }
}

pub(crate) fn map_canonicalization_error(path: &'_ Path) -> impl Fn(std::io::Error) -> Error + '_ {
    move |wrapped| {
        let path = path.to_path_buf();
        match wrapped.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound { path },
            _ => Error::PathCanonicalization { path, wrapped },
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(wrapped: serde_json::Error) -> Self {
        Error::JsonError { wrapped }
    }
}

impl From<toml::de::Error> for Error {
    fn from(wrapped: toml::de::Error) -> Self {
        Error::TomlError { wrapped }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(wrapped: serde_yaml::Error) -> Self {
        Error::YamlError { wrapped }
    }
}

#[doc(hidden)]
pub trait LayeredConfSolid {
    type Layer: LayeredConfLayer + Default + serde::de::DeserializeOwned;
}

#[doc(hidden)]
pub trait LayeredConfLayer {
    type Config: LayeredConfSolid + serde::de::DeserializeOwned;

    fn load_configs(&self) -> Vec<PathBuf>;

    fn default_layer() -> Self;
}

#[doc(hidden)]
pub trait LayeredConfMerge<TLayer> {
    fn merge_from(&mut self, other: &TLayer);
}

#[doc(hidden)]
pub trait LayeredConfSolidify<TSolid> {
    fn solidify(&self) -> Result<TSolid>;
}
