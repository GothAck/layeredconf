impl layeredconf::LayeredConfSolid for Test {
    type Layer = TestLayer;
}
impl layeredconf::LayeredConfLayer for TestLayer {
    type Config = Test;
}
#[derive(serde :: Deserialize, serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct TestLayer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bool")]
    boolean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    integer: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    optional: Option<String>,
}
impl std::default::Default for TestLayer {
    fn default() -> Self {
        Self {
            boolean: None,
            integer: None,
            optional: None,
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
        if self.optional.is_none() {
            self.optional = other.optional.clone();
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
        let optional = self.optional.clone();
        if !missing.is_empty() {
            return Err(layeredconf::Error::SolidifyFailedMissing { missing });
        }
        Ok(Test {
            boolean: boolean.unwrap(),
            integer: integer.unwrap(),
            optional,
        })
    }
}
