#[derive(serde :: Deserialize, serde :: Serialize, clap :: Parser, Clone, Debug)]
struct TestLayer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "TestSubConfigLayer :: empty")]
    #[clap(flatten)]
    subconfig: TestSubConfigLayer,
}
impl layeredconf::LayeredConfSolid for Test {
    type Layer = TestLayer;
}
impl layeredconf::LayeredConfLayer for TestLayer {
    type Config = Test;
    fn load_configs(&self) -> Vec<std::path::PathBuf> {
        let mut load_configs = vec![];
        load_configs
    }
    fn default_layer() -> Self {
        Self {
            name: None,
            subconfig: TestSubConfigLayer::default_layer(),
        }
    }
}
impl TestLayer {
    fn empty(&self) -> bool {
        let mut empty = vec![];
        empty.push(self.name.is_none());
        empty.push(self.subconfig.empty());
        empty.iter().all(|v| *v)
    }
}
impl std::default::Default for TestLayer {
    fn default() -> Self {
        Self {
            name: None,
            subconfig: TestSubConfigLayer::default(),
        }
    }
}
impl layeredconf::LayeredConfMerge<TestLayer> for TestLayer {
    fn merge_from(&mut self, other: &TestLayer) {
        if self.name.is_none() {
            self.name = other.name.clone();
        }
        self.subconfig.merge_from(&other.subconfig);
    }
}
impl layeredconf::LayeredConfSolidify<Test> for TestLayer {
    fn solidify(&self) -> layeredconf::Result<Test> {
        let mut missing = vec![];
        let name;
        if let Some(val) = &self.name {
            name = Some(val.clone());
        } else {
            name = None;
            missing.push("name".to_string());
        }
        let subconfig = self.subconfig.solidify()?;
        if !missing.is_empty() {
            return Err(layeredconf::Error::SolidifyFailedMissing { missing });
        }
        Ok(Test {
            name: name.unwrap(),
            subconfig,
        })
    }
}
