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
            boolean: None,
            integer: None,
        }
    }
}
#[derive(serde :: Deserialize, serde :: Serialize, clap :: Parser, Clone, Debug)]
struct TestLayer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boolean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    integer: Option<u64>,
}
impl TestLayer {
    fn empty(&self) -> bool {
        let mut empty = vec![];
        empty.push(self.boolean.is_none());
        empty.push(self.integer.is_none());
        empty.iter().all(|v| *v)
    }
}
impl std::default::Default for TestLayer {
    fn default() -> Self {
        Self {
            boolean: None,
            integer: None,
        }
    }
}
impl layeredconf::LayeredConfMerge<TestLayer> for TestLayer {
    fn merge_from(&mut self, other: &TestLayer) {
        if self.boolean.is_none() {
            self.boolean = other.boolean.clone();
        }
        if self.integer.is_none() {
            self.integer = other.integer.clone();
        }
    }
}
impl layeredconf::LayeredConfSolidify<Test> for TestLayer {
    fn solidify(&self) -> layeredconf::Result<Test> {
        let mut missing = vec![];
        let boolean;
        if let Some(val) = &self.boolean {
            boolean = Some(val.clone());
        } else {
            boolean = None;
            missing.push("boolean".to_string());
        }
        let integer;
        if let Some(val) = &self.integer {
            integer = Some(val.clone());
        } else {
            integer = None;
            missing.push("integer".to_string());
        }
        if !missing.is_empty() {
            return Err(layeredconf::Error::SolidifyFailedMissing { missing });
        }
        Ok(Test {
            boolean: boolean.unwrap(),
            integer: integer.unwrap(),
        })
    }
}
