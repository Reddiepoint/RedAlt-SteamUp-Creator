use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::modules::compression::{Archiver, CompressionSettings};

#[derive(Deserialize, Serialize)]
pub struct SevenZipSettings {
    pub path: Option<PathBuf>,
}

impl Default for SevenZipSettings {
    fn default() -> Self {
        Self {
            path: {
                let paths = CompressionSettings::get_detected_paths();
                let mut archiver_path = String::new();
                for path in paths.into_iter().flatten() {
                    if path.contains("7zFM.exe") {
                        archiver_path = path;
                    }
                }
                if !archiver_path.is_empty() {
                    Some(archiver_path.into())
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WinRARSettings {
    pub path: Option<PathBuf>,
}

impl Default for WinRARSettings {
    fn default() -> Self {
        Self {
            path: {
                let paths = CompressionSettings::get_detected_paths();
                let mut archiver_path = String::new();
                for path in paths.into_iter().flatten() {
                    if path.contains("WinRAR.exe") {
                        archiver_path = path;
                    }
                }
                if !archiver_path.is_empty() {
                    Some(archiver_path.into())
                } else {
                    None
                }
            }
        }
    }
}