use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::thread;
use crossbeam_channel::Sender;
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use crate::modules::compression_settings::{SevenZipSettings, WinRARSettings};
use crate::modules::depot_downloader::OSType;

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum Archiver {
    SevenZip,
    WinRAR,
    // Zip
}

impl Display for Archiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Archiver::SevenZip => "7zip",
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

        let keys = [(format!("{}\\7zFM.exe", subkey), "7z.exe"), (format!("{}\\WinRAR.exe", subkey), "\\Rar.exe")];
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
        println!("{:?}", paths);
        paths
    }

    pub fn compress_files(&self, stdo_sender: Sender<String>) {
        let archiver = self.archiver.clone();
        let seven_zip_settings = self.seven_zip_settings.clone();
        let win_rar_settings = self.win_rar_settings.clone();
        let path = self.download_path.clone();
        thread::spawn(move || {
            let result = match archiver {
                Archiver::SevenZip => seven_zip_settings.compress(path, stdo_sender.clone()),
                Archiver::WinRAR => win_rar_settings.compress(path, stdo_sender.clone()),
            };

            match result {
                Ok(_) => {
                    let _ = stdo_sender.send("\nFinished compressing files.\n".to_string());
                }
                Err(error) => {
                    let _ = stdo_sender.send(format!("\nFailed to compress files: {}.\n", error));
                }
            }
        });
    }
}

#[test]
fn test_get_paths() {
    CompressionSettings::get_detected_paths();
}