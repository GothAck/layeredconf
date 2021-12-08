use std::collections::HashMap;

use layeredconf::{Builder, Format, LayeredConf, Source};
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
    builder.new_layer(Source::File(
        "examples/layers/lowest.yaml".parse()?,
        Format::Auto,
    ));
    builder.new_layer(Source::File(
        "examples/layers/mid.toml".parse()?,
        Format::Auto,
    ));
    builder.new_layer(Source::File(
        "examples/layers/highest.json".parse()?,
        Format::Auto,
    ));
    builder.new_layer(Source::FileOptional(
        "examples/layers/does_not_exist.json".parse()?,
        Format::Auto,
    ));
    builder.new_layer(Source::Arguments);
    let solid = builder.solidify()?;

    println!("{:?}", solid);

    Ok(())
}
