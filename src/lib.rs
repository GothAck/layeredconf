pub mod layers;

use std::path::{Path, PathBuf};

use thiserror::Error as ThisError;

pub use layeredconf_derive::LayeredConf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Solidify failed, missing fields {missing:?}")]
    SolidifyFailedMissing { missing: Vec<String> },
    #[error("Solidify failed, no layers")]
    SolidifyFailedNoLayers,
    #[error("Unknown extension {extension:?}")]
    UnknownExtension { extension: Option<String> },
    #[error("Auto format detection failed")]
    AutoFormatFailed,
    #[error("Loop detected loading config files")]
    LoopingLoadConfig,
    #[error("Path canonicalization error {wrapped:?} for {path:?}")]
    PathCanonicalization {
        wrapped: std::io::Error,
        path: PathBuf,
    },
    #[error("I/O Error {wrapped:?} for {path:?}")]
    IoError {
        wrapped: std::io::Error,
        path: PathBuf,
    },
    #[error("Json Error {wrapped:?}")]
    JsonError { wrapped: serde_json::Error },
    #[error("Toml Error {wrapped:?}")]
    TomlError { wrapped: toml::de::Error },
    #[error("Yaml Error {wrapped:?}")]
    YamlError { wrapped: serde_yaml::Error },
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

pub trait LayeredConfSolid {
    type Layer: LayeredConfLayer + Default + serde::de::DeserializeOwned;
}

pub trait LayeredConfLayer {
    type Config: LayeredConfSolid + serde::de::DeserializeOwned;

    fn load_configs(&self) -> Vec<PathBuf>;
}

pub trait LayeredConfMerge<TLayer> {
    fn merge_from(&mut self, other: &TLayer);
}

pub trait LayeredConfSolidify<TSolid> {
    fn solidify(&self) -> Result<TSolid>;
}
