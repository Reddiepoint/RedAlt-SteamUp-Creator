use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Changes {
    pub app: String,
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
