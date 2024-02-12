use std::ffi::OsStr;
use std::path::Path;
use eframe::egui::{ComboBox, Context, Slider, TextEdit, Ui};
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use crate::modules::compression::{Archiver, CompressionSettings};
use crate::modules::depot_downloader::{DepotDownloaderSettings, OSType};

#[derive(Default, Deserialize, Serialize)]
pub struct SettingsUI {
    pub depot_downloader_settings: DepotDownloaderSettings,
    pub compression_settings: CompressionSettings,
    #[serde(skip)]
    pub read_settings: bool,
}

impl SettingsUI {
    pub fn read_settings(&mut self) {
        if self.read_settings {
            return;
        }

        if let Ok(mut file) = std::fs::File::open("settings.json") {
            self.read_settings = true;
            let settings: SettingsUI = serde_json::from_reader(&mut file).unwrap_or_default();
            self.depot_downloader_settings = settings.depot_downloader_settings;
        }
    }

    fn set_settings(&mut self) {
        let mut username = String::new();
        if !self.depot_downloader_settings.remember_credentials {
            username = self.depot_downloader_settings.username.clone();
            self.depot_downloader_settings.username = "".to_string();
        }
        let _ = std::fs::write("settings.json", serde_json::to_string_pretty(&self).unwrap());
        self.depot_downloader_settings.username = username;
    }

    pub fn display(ctx: &Context, ui: &mut Ui, settings_ui: &mut SettingsUI) {
        settings_ui.display_settings_buttons(ui);
        settings_ui.display_depot_downloader_settings(ui);
        ui.separator();
        settings_ui.display_compression_settings(ui);
        // let _ = std::fs::write("last_user.txt", settings_ui.depot_downloader_settings.username.clone());
    }

    fn display_settings_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Save config").clicked() {
                self.set_settings();
            }

            if ui.button("Reload config").clicked() {
                self.read_settings = false;
            }
        });
    }
    fn display_depot_downloader_settings(&mut self, ui: &mut Ui) {
        ui.heading("Steam Depot Downloader Settings");
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.depot_downloader_settings.username);
        });
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(TextEdit::singleline(&mut self.depot_downloader_settings.password)
                .password(true));
        });
        ui.checkbox(&mut self.depot_downloader_settings.remember_credentials,
                    "Remember credentials (Requires login with Depot Downloader at least once. \
                    Subsequent logins require the username only.)");

        ui.horizontal(|ui| {
            ui.label("Download files for OS:");
            ComboBox::from_id_source("OS").selected_text(format!("{}", self.depot_downloader_settings.os))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Windows, "Windows");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Linux, "Linux");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Mac, "Mac");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Current, "Current OS");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Max number of server connections:");
            ui.add(Slider::new(&mut self.depot_downloader_settings.max_servers, 1..=32));
        });

        ui.horizontal(|ui| {
            ui.label("Max number of concurrent chunks downloaded:");
            ui.add(Slider::new(&mut self.depot_downloader_settings.max_downloads, 1..=32));
        });
    }

    fn display_compression_settings(&mut self, ui: &mut Ui) {
        ui.heading("Compression Settings");
        ui.horizontal(|ui| {
            ui.label("Archiver:");
            ComboBox::from_id_source("Archiver").selected_text(format!("{}", self.compression_settings.archiver))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.compression_settings.archiver, Archiver::SevenZip, "7zip");
                    ui.selectable_value(&mut self.compression_settings.archiver, Archiver::WinRAR, "WinRAR");
                });
        });

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                let path = match self.compression_settings.archiver {
                    Archiver::SevenZip => &self.compression_settings.seven_zip_settings.path,
                    Archiver::WinRAR => &self.compression_settings.win_rar_settings.path,
                };

                match &path {
                    None => ui.label("Select archiver executable:"),
                    Some(path) => ui.label(format!("Using archiver: {}", path.display())),
                };

                if ui.button("Change path").clicked() {
                    // Show only files with the extension "json".
                    let filter = Box::new({
                        let ext = Some(OsStr::new("exe"));
                        move |path: &Path| -> bool { path.extension() == ext }
                    });
                    let mut dialog = FileDialog::open_file(path.clone()).show_files_filter(filter);
                    dialog.open();
                    self.compression_settings.open_archiver_dialog = Some(dialog);
                }

                if let Some(dialog) = &mut self.compression_settings.open_archiver_dialog {
                    if dialog.show(ui.ctx()).selected() {
                        if let Some(file) = dialog.path() {
                            match self.compression_settings.archiver {
                                Archiver::SevenZip => {
                                    self.compression_settings.seven_zip_settings.path = Some(file.to_path_buf()).clone();
                                }
                                Archiver::WinRAR => {
                                    self.compression_settings.win_rar_settings.path = Some(file.to_path_buf()).clone();
                                }
                            }
                        }
                    }
                }
            });
        });
    }
}

