use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

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
    layers: Vec<Arc<Layer<TSolid::Layer>>>,
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

    pub fn new_layer(&mut self, source: Source) -> Arc<Layer<TSolid::Layer>> {
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
            let obj = layer.obj.lock().unwrap();
            merged.merge_from(&obj);
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
pub struct Layer<TLayer>
where
    TLayer: LayeredConfLayer
        + std::fmt::Debug
        + Default
        + serde::de::DeserializeOwned
        + clap::FromArgMatches
        + clap::IntoApp
        + clap::Parser
        + Sized,
{
    source: Source,
    obj: Mutex<TLayer>,
    loaded: AtomicBool,
}

impl<TLayer> Layer<TLayer>
where
    TLayer: LayeredConfLayer
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
            obj: Mutex::from(TLayer::default()),
            loaded: AtomicBool::new(false),
        }
    }

    pub fn load(&self) -> super::Result<()> {
        *self.obj.lock().unwrap() = match &self.source {
            Source::File { path, format } => {
                let string = std::fs::read_to_string(path)?;
                match format {
                    Format::Json => serde_json::from_str(&string)?,
                    Format::Toml => toml::from_str(&string)?,
                    Format::Yaml => serde_yaml::from_str(&string)?,
                }
            }
            Source::FileOptional { path, format } => match std::fs::read_to_string(path) {
                Ok(string) => match format {
                    Format::Json => serde_json::from_str(&string)?,
                    Format::Toml => toml::from_str(&string)?,
                    Format::Yaml => serde_yaml::from_str(&string)?,
                },
                Err(_) => TLayer::default(),
            },
            Source::String { str, format } => match format {
                Format::Json => serde_json::from_str(str)?,
                Format::Toml => toml::from_str(str)?,
                Format::Yaml => serde_yaml::from_str(str)?,
            },
            Source::Environment { prefix: _ } => {
                unimplemented!();
            }
            Source::Arguments => TLayer::parse(),
        };
        self.loaded.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[derive(Debug)]
pub enum Format {
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
}
