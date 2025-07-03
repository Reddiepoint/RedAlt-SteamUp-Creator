use crate::modules::compression_settings::{SevenZipSettings, WinRARSettings};
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use which::which;

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum Archiver {
    SevenZip,
    WinRAR,
    // Zip
}

impl Display for Archiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Archiver::SevenZip => "7zip",
                Archiver::WinRAR => "WinRAR",
            }
        )
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CompressorSettings {
    pub download_path: PathBuf,
    pub archiver: Archiver,
    #[serde(skip)]
    pub open_archiver_dialog: Option<FileDialog>,
    #[serde(skip)]
    pub archiver_path: Option<PathBuf>,
    pub seven_zip_settings: SevenZipSettings,
    pub win_rar_settings: WinRARSettings,
    pub multiup_direct_path: Option<PathBuf>,
    #[serde(skip)]
    pub multiup_direct_file_dialog: Option<FileDialog>,
}

impl Default for CompressorSettings {
    fn default() -> Self {
        Self {
            download_path: current_dir().unwrap().to_path_buf(),
            archiver: {
                let paths = CompressorSettings::get_detected_paths();
                let mut archiver = Archiver::SevenZip;
                for path in paths.into_iter().flatten() {
                    if path.contains("7z") {
                        archiver = Archiver::SevenZip;

                        break; // Use 7zip if possible instead of WinRAR
                    } else if path.contains("WinRAR") {
                        archiver = Archiver::WinRAR;
                    }
                }
                archiver
            },
            open_archiver_dialog: None,
            archiver_path: None,
            seven_zip_settings: SevenZipSettings::default(),
            win_rar_settings: WinRARSettings::default(),
            multiup_direct_path: {
                let mut executable = None;
                for file in current_dir().unwrap().read_dir().unwrap().flatten() {
                    if file
                        .file_name()
                        .to_str()
                        .unwrap()
                        .contains("Multiup-Direct")
                    {
                        executable = Some(file.path());
                        break;
                    }
                }
                executable
            },
            multiup_direct_file_dialog: None,
        }
    }
}

impl CompressorSettings {
    pub fn get_detected_paths() -> Vec<Option<String>> {
        let mut paths: Vec<Option<String>> = Vec::new();
        let compressors = ["7z", "WinRAR"];
        for compressor in compressors {
            if let Ok(path) = which(compressor) {
                paths.push(Some(path.to_str().unwrap().to_string()));
            }
        }
        paths
    }
}

// pub fn compress_files(archiver: Archiver,
//                       download_path: String,
//                       seven_zip_settings: SevenZipSettings,
//                       win_rar_settings: WinRARSettings,
//                       input_window_opened_sender: Sender<bool>,
//                       input_receiver: Receiver<String>,
//                       output_sender: Sender<String>,
//                       status_sender: Sender<std::io::Result<()>>) {
//
// }
