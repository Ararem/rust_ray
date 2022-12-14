use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResourcesConfig {
    pub resources_path: String,
    pub fonts_path: String,
}

impl ResourcesConfig {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        Self {
            resources_path: "app_resources".into(),
            fonts_path: "fonts".into(),
        }
    }
}
