use std::{collections::HashMap, path::PathBuf};

use layeredconf::{
    layers::{Builder, Format, Source},
    LayeredConf,
};
use serde::{Deserialize, Serialize};

#[derive(LayeredConf, Deserialize, Serialize, Debug)]
struct Config {
    #[layered(load_config)]
    #[clap(long)]
    config: Option<PathBuf>,
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
        path: PathBuf::from("examples/load_config/lowest.yaml"),
        format: Format::Auto,
    });
    builder.new_layer(Source::File {
        path: PathBuf::from("examples/load_config/mid.toml"),
        format: Format::Auto,
    });
    builder.new_layer(Source::File {
        path: PathBuf::from("examples/load_config/highest.json"),
        format: Format::Auto,
    });
    builder.new_layer(Source::FileOptional {
        path: PathBuf::from("examples/load_config/does_not_exist.json"),
        format: Format::Auto,
    });
    builder.new_layer(Source::Arguments);
    let solid = builder.solidify()?;

    println!("{:?}", solid);

    Ok(())
}
