use std::{collections::HashMap, path::PathBuf};

use layeredconf::{
    layers::{Builder, Format, Source},
    LayeredConf,
};
use serde::{Deserialize, Serialize};

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
struct Config {
    #[clap(long)]
    name: String,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    optional: Option<String>,

    #[clap(long)]
    vec: Vec<String>,

    #[layered(subconfig)]
    twitter: Twitter,
    #[layered(subconfig)]
    db: Database,
}

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
#[layered(subconfig)]
struct Twitter {
    auth_token: String,
    rate_limit: Option<u32>,
}

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
#[layered(subconfig)]
struct Database {
    uri: String,
    #[clap(skip)]
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
    builder.new_layer(Source::FileOptional {
        path: PathBuf::from("examples/layers/does_not_exist.json"),
        format: Format::Json,
    });
    builder.new_layer(Source::Arguments);
    let solid = builder.solidify()?;

    println!("{:?}", solid);

    Ok(())
}
