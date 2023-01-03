use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesConfig {
    pub resources_path: String,
    pub fonts_path: String,
}

impl ResourcesConfig {
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
