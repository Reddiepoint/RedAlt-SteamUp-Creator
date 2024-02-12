use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub enum Archiver {
    SevenZip(String),
    WinRAR(String),
    Zip
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CompressionSettings {
    pub download_path: String,
    pub archiver: Archiver
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            download_path: String::new(),
            archiver: Archiver::Zip,
        }
    }
}
