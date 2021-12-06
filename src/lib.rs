#![deny(missing_docs)]

//!# LayeredConf
//!
//!## Yet Another Config Package
//!
//!Hopefully this one will be useful to someone. Incoming features:
//!- More Documentation
//!- Features

pub mod layers;

use std::path::{Path, PathBuf};

use thiserror::Error as ThisError;

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
    move |wrapped| Error::IoError {
        wrapped,
        path: path.to_path_buf(),
    }
}

pub(crate) fn map_canonicalization_error(path: &'_ Path) -> impl Fn(std::io::Error) -> Error + '_ {
    move |wrapped| Error::IoError {
        wrapped,
        path: path.to_path_buf(),
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
}

#[doc(hidden)]
pub trait LayeredConfMerge<TLayer> {
    fn merge_from(&mut self, other: &TLayer);
}

#[doc(hidden)]
pub trait LayeredConfSolidify<TSolid> {
    fn solidify(&self) -> Result<TSolid>;
}
