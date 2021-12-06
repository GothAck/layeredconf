use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use clap::Parser;

use crate::Error;

use super::{LayeredConfLayer, LayeredConfMerge, LayeredConfSolid, LayeredConfSolidify, Result};

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
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn new_layer(&mut self, source: Source) -> Arc<Layer<TSolid>> {
        let layer = Arc::from(Layer::new(source));
        self.layers.push(layer.clone());
        layer
    }

    pub fn load_all(&self) -> Result<()> {
        for layer in self.layers.iter() {
            layer.load()?;
        }
        Ok(())
    }

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
pub struct Layer<TSolid>
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
                let path = self.canonicalize(path)?;
                if seen_paths.contains(&path) {
                    return Err(Error::LoopingLoadConfig);
                }

                let string = std::fs::read_to_string(&path)?;
                match self.auto_format(&path, format)? {
                    Format::Auto => return Err(Error::AutoFormatFailed),
                    Format::Json => serde_json::from_str(&string)?,
                    Format::Toml => toml::from_str(&string)?,
                    Format::Yaml => serde_yaml::from_str(&string)?,
                }
            }
            Source::FileOptional { path, format } => {
                let path = self.canonicalize(path)?;
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

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        std::fs::canonicalize(path).map_err(|wrapped| Error::IoError { wrapped })
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

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Auto,
    Json,
    Toml,
    Yaml,
}

#[derive(Debug)]
pub enum Source {
    File { path: PathBuf, format: Format },
    FileOptional { path: PathBuf, format: Format },
    String { str: String, format: Format },
    Environment { prefix: Option<String> },
    Arguments,
    ArgumentsFrom(Vec<String>),
}
