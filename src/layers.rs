

use std::{sync::{Arc, RwLock}, path::PathBuf};

use crate::Error;

use super::{LayeredConfLayer, LayeredConfSolid, LayeredConfMerge, LayeredConfSolidify, Result};

#[derive(Debug)]
pub struct Builder<TSolid> where TSolid: LayeredConfSolid, <TSolid>::Layer: LayeredConfLayer + std::fmt::Debug + Default + serde::de::DeserializeOwned {
    layers: Vec<Arc<Layer<TSolid::Layer>>>,
}

impl<TSolid> Builder<TSolid> where TSolid: LayeredConfSolid, <TSolid>::Layer: LayeredConfLayer + LayeredConfMerge<<TSolid>::Layer> + LayeredConfSolidify<TSolid> + std::fmt::Debug + Default + serde::de::DeserializeOwned {
    pub fn new() -> Self {
        Self {
            layers: vec![],
        }
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
            if !*layer.loaded.read().unwrap() {
                layer.load()?;
            }
        }
        for (i, layer) in self.layers[0..self.layers.len() - 1].iter().enumerate().rev() {
            let obj_plus_one = self.layers[i + 1].obj.write().unwrap();
            let mut obj = layer.obj.write().unwrap();
            obj.merge_from(&obj_plus_one);
        }

        self.layers[0].obj.read().unwrap().solidify()
    }
}

#[derive(Debug)]
pub struct Layer<TLayer> where TLayer: LayeredConfLayer + std::fmt::Debug + Default + serde::de::DeserializeOwned {
    source: Source,
    obj: RwLock<TLayer>,
    loaded: RwLock<bool>,
}

impl<TLayer> Layer<TLayer> where TLayer: LayeredConfLayer + std::fmt::Debug + Default + serde::de::DeserializeOwned {
    fn new(source: Source) -> Self {
        Self {
            source,
            obj: RwLock::from(TLayer::default()),
            loaded: RwLock::from(false),
        }
    }

    pub fn load(&self) -> super::Result<()> {
        *self.obj.write().unwrap() =
            match &self.source {
                Source::File { path, format } => match format {
                    Format::Json => {
                        serde_json::from_reader(std::fs::File::open(path)?)?
                    },
                    Format::Toml => {
                        toml::from_str(&std::fs::read_to_string(path)?)?
                    },
                    Format::Yaml => {
                        serde_yaml::from_reader(std::fs::File::open(path)?)?
                    },
                },
                Source::String { str, format } => match format {
                    Format::Json => {
                        serde_json::from_str(str)?
                    },
                    Format::Toml => {
                        toml::from_str(str)?
                    },
                    Format::Yaml => {
                        serde_yaml::from_str(str)?
                    },
                },
                Source::Environment { prefix: _ } => {
                    unimplemented!();
                },
                Source::Arguments => {
                    unimplemented!();
                }
            };
        *self.loaded.write().unwrap() = true;
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
    File {
        path: PathBuf,
        format: Format,
    },
    String {
        str: String,
        format: Format,
    },
    Environment {
        prefix: Option<String>,
    },
    Arguments,
}
