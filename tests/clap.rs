use std::time::Duration;

use clap::IntoApp;
use serde::{Deserialize, Serialize};

use layeredconf::LayeredConf;

#[derive(LayeredConf, Deserialize, Serialize, Clone, Debug)]
/// Clap app description
///
/// Long description.
struct Config {
    #[clap(long)]
    /// Optional string
    ///
    /// Longer description...
    optional: Option<String>,
    #[layered(subconfig)]
    /// Subconfig
    subconfig: SubConfig,
}

#[derive(LayeredConf, Deserialize, Serialize, Clone, Debug)]
#[layered(subconfig)]
struct SubConfig {
    #[clap(long)]
    /// Flibble
    flibble: u64,
    #[clap(skip)]
    /// Duration
    duration: Duration,
}

#[test]
fn test_clap_help_via_app() {
    let mut app = ConfigLayer::into_app();

    let mut output = Vec::new();
    app.write_long_help(&mut output).unwrap();
    let string = String::from_utf8(output).unwrap();

    assert!(string.contains("Clap app description"));
    assert!(string.contains("Long description."));
    assert!(string.contains("Optional string"));
    assert!(string.contains("Longer description..."));
    assert!(string.contains("Flibble"));

    // These fields shouldn't show up in help
    assert!(!string.contains("Subconfig"));
    assert!(!string.contains("Duration"));
}
