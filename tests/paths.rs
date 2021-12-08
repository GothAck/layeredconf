use std::{env::current_dir, path::PathBuf};

use serde::{Deserialize, Serialize};

use layeredconf::{Builder, Error, Format, LayeredConf, Result, Source};

#[derive(LayeredConf, Deserialize, Serialize, Clone, Debug)]
struct Config {
    #[layered(load_config)]
    #[clap(long)]
    config: Option<PathBuf>,
    #[clap(long)]
    name: String,
}

#[test]
fn test() -> anyhow::Result<()> {
    let current_dir = current_dir()?;
    let arg_config = current_dir.join("tests/paths/arg/config.yaml");
    let arg_str = arg_config.to_str().unwrap();

    let args: Vec<String> = vec!["paths", "--config", arg_str]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let config: Config = Builder::new()
        .new_layer(Source::File {
            path: PathBuf::from("./tests/paths/config.yaml"),
            format: Format::Auto,
        })
        .new_layer(Source::ArgumentsFrom(args))
        .solidify()?;

    assert_eq!(config.name, "paths/arg/config.yaml");

    Ok(())
}

#[test]
fn test_loop() -> anyhow::Result<()> {
    let source = Source::File {
        path: PathBuf::from("./tests/paths/loop/config.yaml"),
        format: Format::Auto,
    };

    let result: Result<Config> = Builder::new().new_layer(source.clone()).solidify();

    assert!(matches!(result, Err(Error::LoopingLoadConfig { .. })));

    if let Err(Error::LoopingLoadConfig { path, parents }) = result {
        assert_eq!(path, current_dir()?.join("tests/paths/loop/config.yaml"));
        assert_eq!(
            parents,
            vec![
                Source::File {
                    path: PathBuf::from("other_config.yaml"),
                    format: Format::Auto
                },
                source,
            ],
        );
    }

    Ok(())
}
