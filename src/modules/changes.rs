use std::sync::Arc;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Changes {
    pub depot: String,
    pub initial_build: String,
    pub final_build: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
    pub manifest: String
}

impl Changes {
    pub fn new_error(error: String) -> Changes {
        Changes {
            depot: "Failed to parse JSON".to_string(),
            initial_build: "".to_string(),
            final_build: "".to_string(),
            added: vec![],
            removed: vec![],
            modified: vec![],
            manifest: error,
        }
    }
}