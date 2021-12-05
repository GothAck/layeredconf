use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

use layeredconf::{layers::*, LayeredConf, LayeredConfMerge, LayeredConfSolidify};

#[derive(LayeredConf, Deserialize, Serialize, Clone, Debug)]
struct Config {
    config: String,
    name: String,
    data_path: PathBuf,
    optional: Option<String>,
    #[layered(subconfig)]
    subconfig: SubConfig,
}

#[derive(LayeredConf, Deserialize, Serialize, Clone, Debug)]
struct SubConfig {
    flibble: u64,
    duration: Duration,
}

#[test]
fn test_deser() {
    let input = r#"
config = "Hello"
name = "World"
data_path = "/tmp/rar"
subconfig = { flibble = 1997,  duration = { secs = 10, nanos = 0 } }
"#;
    let _: Config = toml::from_str(input).unwrap();
}

#[test]
fn test_generated_object_defaults() {
    let config_layer = ConfigLayer::default();

    assert_eq!(serde_json::to_string(&config_layer).unwrap(), "{}");
}

static INTERMED_JSON: &str = r#"{ "config": "string", "data_path": "/tmp/path", "subconfig": { "duration": { "secs": 50, "nanos": 99 } } }"#;
static TOP_JSON: &str = r#"{ "name": "yes", "subconfig": { "flibble": 10 } }"#;
static FULL_JSON: &str = r#"{"config":"string","name":"yes","data_path":"/tmp/path","subconfig":{"flibble":10,"duration":{"secs":50,"nanos":99}}}"#;
static FULL_JSON_SOLID: &str = r#"{"config":"string","name":"yes","data_path":"/tmp/path","optional":null,"subconfig":{"flibble":10,"duration":{"secs":50,"nanos":99}}}"#;

#[test]
fn test_merge() {
    let mut base_layer = ConfigLayer::default();

    let mut intermediate_layer: ConfigLayer = serde_json::from_str(INTERMED_JSON).unwrap();

    let top_layer: ConfigLayer = serde_json::from_str(TOP_JSON).unwrap();

    intermediate_layer.merge_from(&top_layer);
    base_layer.merge_from(&intermediate_layer);

    assert_eq!(serde_json::to_string(&base_layer).unwrap(), FULL_JSON);

    let solid = base_layer.solidify().unwrap();
    assert_eq!(solid.config, "string");
    assert_eq!(solid.name, "yes");
    assert_eq!(solid.data_path, PathBuf::from("/tmp/path"));
    assert_eq!(solid.subconfig.flibble, 10);
    assert_eq!(
        solid.subconfig.duration,
        Duration::from_secs(50) + Duration::from_nanos(99)
    );
}

#[test]
fn test_layers() {
    let mut builder = Builder::<Config>::new();
    builder.new_layer(Source::String {
        str: "{}".to_string(),
        format: Format::Json,
    });
    builder.new_layer(Source::String {
        str: INTERMED_JSON.to_string(),
        format: Format::Json,
    });
    builder.new_layer(Source::String {
        str: TOP_JSON.to_string(),
        format: Format::Json,
    });

    builder.load_all().unwrap();

    let solid = builder.solidify().unwrap();

    assert_eq!(serde_json::to_string(&solid).unwrap(), FULL_JSON_SOLID);
}
