use std::path::PathBuf;
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
    pub fn parse_changes(path: &Option<PathBuf>) -> Option<Changes> {
        let changes = match &path {
            Some(file) => {
                match std::fs::read_to_string(file) {
                    Ok(changes) => changes,
                    Err(error) => {
                        println!("Error reading changes file: {}", error);
                        return None;
                    }
                }
            },
            None => {
                println!("Provide a changes file.");
                return None;
            }
        };
        match serde_json::from_str::<Changes>(&changes) {
            Ok(changes) => Some(changes),
            Err(error) => {
                println!("Error parsing changes file: {}", error);
                None
            }
        }
    }
}