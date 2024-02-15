use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use crate::modules::compression_settings::{SevenZipSettings, WinRARSettings};

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum Archiver {
    SevenZip,
    WinRAR,
    // Zip
}

impl Display for Archiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Archiver::SevenZip => "7-zip",
            Archiver::WinRAR => "WinRAR"
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct CompressionSettings {
    pub download_path: String,
    pub archiver: Archiver,
    #[serde(skip)]
    pub open_archiver_dialog: Option<FileDialog>,
    #[serde(skip)]
    pub archiver_path: Option<PathBuf>,
    pub seven_zip_settings: SevenZipSettings,
    pub win_rar_settings: WinRARSettings,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            download_path: String::new(),
            archiver: {
                let paths = CompressionSettings::get_detected_paths();
                let mut archiver = Archiver::SevenZip;
                for path in paths.into_iter().flatten() {
                    if path.contains("7z.exe") {
                        archiver = Archiver::SevenZip;

                        break; // Use 7zip if possible instead of WinRAR
                    } else if path.contains("WinRAR.exe") {
                        archiver = Archiver::WinRAR;
                    }
                }
                archiver
            },
            open_archiver_dialog: None,
            archiver_path: None,
            seven_zip_settings: SevenZipSettings::default(),
            win_rar_settings: WinRARSettings::default(),
        }
    }
}

impl CompressionSettings {
    pub fn get_detected_paths() -> Vec<Option<String>> {
        let mut paths: Vec<Option<String>> = Vec::new();
        // Try to find 7zip in the registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";

        let keys = [(format!("{}\\7zFM.exe", subkey), "7z.exe"), (format!("{}\\WinRAR.exe", subkey), "\\WinRAR.exe")];
        for (key, file) in keys {
            if let Ok(key) = hklm.open_subkey(key) {
                if let Ok(path) = key.get_value("Path") {
                    let mut path: String = path;
                    path += file;
                    paths.push(Some(path));
                } else {
                    paths.push(None);
                }
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