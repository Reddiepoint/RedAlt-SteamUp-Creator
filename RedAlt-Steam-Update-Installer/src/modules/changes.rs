use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Changes {
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub depot: String,
    #[serde(default)]
    pub initial_build: String,
    #[serde(default)]
    pub final_build: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
    #[serde(default)]
    pub manifest: String
}

impl Changes {
    pub fn new_error(error: String) -> Changes {
        Changes {
            app: "Failed to parse JSON".to_string(),
            depot: String::new(),
            initial_build: String::new(),
            final_build: String::new(),
            added: vec![],
            removed: vec![],
            modified: vec![],
            manifest: error,
        }
    }
}