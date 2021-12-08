#[derive(serde :: Deserialize, serde :: Serialize, clap :: Parser, Clone, Debug)]
#[doc = " This is kept so that clap can parse it"]
#[doc = ""]
#[doc = " Long description here."]
struct TestSubConfigLayer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[doc = " This is kept too"]
    #[doc = ""]
    #[doc = " Long description here."]
    test: Option<String>,
}
impl layeredconf::LayeredConfSolid for TestSubConfig {
    type Layer = TestSubConfigLayer;
}
impl layeredconf::LayeredConfLayer for TestSubConfigLayer {
    type Config = TestSubConfig;
    fn load_configs(&self) -> Vec<std::path::PathBuf> {
        let mut load_configs = vec![];
        load_configs
    }
    fn default_layer() -> Self {
        Self { test: None }
    }
}
impl TestSubConfigLayer {
    fn empty(&self) -> bool {
        let mut empty = vec![];
        empty.push(self.test.is_none());
        empty.iter().all(|v| *v)
    }
}
impl std::default::Default for TestSubConfigLayer {
    fn default() -> Self {
        Self { test: None }
    }
}
impl layeredconf::LayeredConfMerge<TestSubConfigLayer> for TestSubConfigLayer {
    fn merge_from(&mut self, other: &TestSubConfigLayer) {
        if self.test.is_none() {
            self.test = other.test.clone();
        }
    }
}
impl layeredconf::LayeredConfSolidify<TestSubConfig> for TestSubConfigLayer {
    fn solidify(&self) -> layeredconf::Result<TestSubConfig> {
        let mut missing = vec![];
        let test;
        if let Some(val) = &self.test {
            test = Some(val.clone());
        } else {
            test = None;
            missing.push("test".to_string());
        }
        if !missing.is_empty() {
            return Err(layeredconf::Error::SolidifyFailedMissing { missing });
        }
        Ok(TestSubConfig {
            test: test.unwrap(),
        })
    }
}
