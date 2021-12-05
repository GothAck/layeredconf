use std::{collections::HashMap, path::PathBuf};

use layeredconf::{
    layers::{Builder, Format, Source},
    LayeredConf,
};
use serde::{Deserialize, Serialize};

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
struct Config {
    name: String,
    port: u16,
    optional: Option<String>,

    vec: Vec<String>,

    #[layered(subconfig)]
    twitter: Twitter,
    #[layered(subconfig)]
    db: Database,
}

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
struct Twitter {
    auth_token: String,
    rate_limit: Option<u32>,
}

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
struct Database {
    uri: String,
    options: HashMap<String, String>,
}

fn main() -> anyhow::Result<()> {
    let mut builder = Builder::<Config>::new();
    builder.new_layer(Source::File {
        path: PathBuf::from("examples/layers/lowest.yaml"),
        format: Format::Yaml,
    });
    builder.new_layer(Source::File {
        path: PathBuf::from("examples/layers/mid.toml"),
        format: Format::Toml,
    });
    builder.new_layer(Source::File {
        path: PathBuf::from("examples/layers/highest.json"),
        format: Format::Json,
    });
    let solid = builder.solidify()?;

    println!("{:?}", solid);

    Ok(())
}
