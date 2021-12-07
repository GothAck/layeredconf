//! Builds a Config from layers

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use clap::Parser;

use crate::{map_canonicalization_error, map_io_error, Error};

use super::{LayeredConfLayer, LayeredConfMerge, LayeredConfSolid, LayeredConfSolidify, Result};

/// Builds a layered configuration
///
/// ```rust
/// use std::path::PathBuf;
///
/// use layeredconf::{Builder, Format, LayeredConf, Source};
///
/// #[derive(LayeredConf, serde::Deserialize)]
/// struct Config {
///     name: String,
/// }
///
/// fn main() -> anyhow::Result<()> {
///     let config: Config = Builder::new()
///         .new_layer(Source::String { str: "{ \"name\": \"test\" }".to_string(), format: Format::Json })
///         .solidify()?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Builder<TSolid>
where
    TSolid: LayeredConfSolid,
    <TSolid>::Layer: LayeredConfLayer
        + LayeredConfMerge<<TSolid>::Layer>
        + LayeredConfSolidify<TSolid>
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    layers: Vec<Arc<Layer<TSolid>>>,
}

impl<TSolid> Builder<TSolid>
where
    TSolid: LayeredConfSolid,
    <TSolid>::Layer: LayeredConfLayer
        + LayeredConfMerge<<TSolid>::Layer>
        + LayeredConfSolidify<TSolid>
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    /// Returns a new Builder
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    /// Adds a new Layer to the Builder from a source
    pub fn new_layer(&mut self, source: Source) -> &mut Self {
        let layer = Arc::from(Layer::new(source));
        self.layers.push(layer);
        self
    }

    /// Loads all the config sources defined in the builder
    pub fn load_all(&self) -> Result<()> {
        for layer in self.layers.iter() {
            layer.load()?;
        }
        Ok(())
    }

    /// Solidifies the Builder ingo a Config
    pub fn solidify(&self) -> Result<TSolid> {
        if self.layers.is_empty() {
            return Err(Error::SolidifyFailedNoLayers);
        }
        for layer in &self.layers {
            if !layer.loaded.load(Ordering::Relaxed) {
                layer.load()?;
            }
        }

        let mut merged = <TSolid>::Layer::default();

        for layer in self.layers.iter().rev() {
            layer.merge_into(&mut merged)?;
        }

        merged.merge_from(&<TSolid>::Layer::default_layer());

        merged.solidify()
    }
}

impl<TSolid> Default for Builder<TSolid>
where
    TSolid: LayeredConfSolid,
    <TSolid>::Layer: LayeredConfLayer
        + LayeredConfMerge<<TSolid>::Layer>
        + LayeredConfSolidify<TSolid>
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Layer<TSolid>
where
    TSolid: LayeredConfSolid,
    <TSolid>::Layer: LayeredConfLayer
        + LayeredConfMerge<<TSolid>::Layer>
        + LayeredConfSolidify<TSolid>
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    source: Source,
    obj: Mutex<<TSolid>::Layer>,
    sub_layers: Mutex<Vec<Layer<TSolid>>>,
    loaded: AtomicBool,
}

impl<TSolid> Layer<TSolid>
where
    TSolid: LayeredConfSolid,
    <TSolid>::Layer: LayeredConfLayer
        + LayeredConfMerge<<TSolid>::Layer>
        + LayeredConfSolidify<TSolid>
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    fn new(source: Source) -> Self {
        Self {
            source,
            obj: Mutex::from(<TSolid>::Layer::default()),
            sub_layers: Mutex::from(Vec::new()),
            loaded: AtomicBool::new(false),
        }
    }

    fn merge_into(&self, merged: &mut <TSolid>::Layer) -> Result<()> {
        let obj = self.obj.lock().unwrap();
        let sub_layers = self.sub_layers.lock().unwrap();

        merged.merge_from(&*obj);
        for sub_layer in sub_layers.iter().rev() {
            sub_layer.merge_into(merged)?;
        }
        Ok(())
    }

    fn load_impl(&self, seen_paths: &mut HashSet<PathBuf>) -> super::Result<()> {
        let mut obj = self.obj.lock().unwrap();
        let mut sub_layers = self.sub_layers.lock().unwrap();

        *obj = match &self.source {
            Source::File { path, format } => {
                let path = self
                    .canonicalize(path)
                    .map_err(map_canonicalization_error(path))?;

                if seen_paths.contains(&path) {
                    return Err(Error::LoopingLoadConfig);
                }

                let string = std::fs::read_to_string(&path).map_err(map_io_error(&path))?;

                match self.auto_format(&path, format)? {
                    Format::Auto => return Err(Error::AutoFormatFailed),
                    Format::Json => serde_json::from_str(&string)?,
                    Format::Toml => toml::from_str(&string)?,
                    Format::Yaml => serde_yaml::from_str(&string)?,
                }
            }
            Source::FileOptional { path, format } => match self.canonicalize(path) {
                Err(error) => match error.kind() {
                    std::io::ErrorKind::NotFound => <TSolid>::Layer::default(),
                    _ => {
                        return Err(Error::IoError {
                            wrapped: error,
                            path: path.to_path_buf(),
                        })
                    }
                },
                Ok(path) => {
                    if seen_paths.contains(&path) {
                        return Err(Error::LoopingLoadConfig);
                    }

                    match std::fs::read_to_string(&path) {
                        Ok(string) => match self.auto_format(&path, format)? {
                            Format::Auto => return Err(Error::AutoFormatFailed),
                            Format::Json => serde_json::from_str(&string)?,
                            Format::Toml => toml::from_str(&string)?,
                            Format::Yaml => serde_yaml::from_str(&string)?,
                        },
                        Err(_) => <TSolid>::Layer::default(),
                    }
                }
            },
            Source::String { str, format } => match format {
                Format::Auto => return Err(Error::AutoFormatFailed),
                Format::Json => serde_json::from_str(str)?,
                Format::Toml => toml::from_str(str)?,
                Format::Yaml => serde_yaml::from_str(str)?,
            },
            Source::Environment { prefix: _ } => {
                unimplemented!();
            }
            Source::Arguments => <TSolid>::Layer::parse(),
            Source::ArgumentsFrom(from) => <TSolid>::Layer::parse_from(from),
        };

        *sub_layers = obj
            .load_configs()
            .iter()
            .cloned()
            .map(|path| {
                Layer::new(Source::File {
                    path,
                    format: Format::Auto,
                })
            })
            .collect();

        for sub_layer in sub_layers.iter() {
            sub_layer.load_impl(seen_paths)?;
        }

        self.loaded.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn auto_format(&self, path: &Path, format: &Format) -> Result<Format> {
        match format {
            Format::Auto => {
                let extension = path.extension().map(|s| s.to_str()).flatten();
                match extension {
                    Some("json") => Ok(Format::Json),
                    Some("toml") => Ok(Format::Toml),
                    Some("yaml") => Ok(Format::Yaml),
                    _ => Err(Error::UnknownExtension {
                        extension: extension.map(|s| s.to_string()),
                    }),
                }
            }
            format => Ok(*format),
        }
    }

    pub fn load(&self) -> super::Result<()> {
        let mut seen_paths = HashSet::new();
        self.load_impl(&mut seen_paths)
    }
}

/// Config file format
#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// Automatically detect config file format from it's extension
    Auto,
    /// JSON formatted
    Json,
    /// TOML formatted
    Toml,
    /// YAML formatted
    Yaml,
}

/// Config source
#[derive(Debug)]
pub enum Source {
    /// From a file
    File {
        /// File path
        path: PathBuf,
        /// File format
        format: Format,
    },
    /// From a file, ignoring if it doesn't exist
    FileOptional {
        /// File path
        path: PathBuf,
        /// File format
        format: Format,
    },
    /// From a String
    String {
        /// String contents
        str: String,
        /// Format
        format: Format,
    },
    /// From process env (currently unimplemented)
    Environment {
        /// Optional prefix for environment variables
        prefix: Option<String>,
    },
    /// From argv
    Arguments,
    /// From an Vec of arguments
    ArgumentsFrom(Vec<String>),
}
