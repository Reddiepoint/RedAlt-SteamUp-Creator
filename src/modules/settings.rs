use std::ffi::OsStr;
use std::path::Path;
use eframe::egui::{ComboBox, Context, Slider, TextEdit, Ui};
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use crate::modules::compression::{Archiver, CompressionSettings};
use crate::modules::compression_settings::SevenZipSettings;
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
        username = self.depot_downloader_settings.username.clone();
        if !self.depot_downloader_settings.remember_credentials {
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
        match self.compression_settings.archiver {
            Archiver::SevenZip => {
                ui.horizontal(|ui| {
                    ui.label("Password");
                    ui.text_edit_singleline(&mut self.compression_settings.seven_zip_settings.password);
                });

                ui.horizontal(|ui| {
                    ui.label("Archive format:");
                    ComboBox::from_id_source("Format").selected_text(self.compression_settings.seven_zip_settings.archive_format.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.archive_format, "7z".to_string(), "7z");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Compression level:");
                    ComboBox::from_id_source("Compression Level").selected_text(format!("{}", self.compression_settings.seven_zip_settings.compression_level))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 0, "0 - Store");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 1, "1 - Fastest");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 3, "3 - Fast");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 5, "5 - Normal");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 7, "7 - Maximum");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_level, 9, "9 - Ultra");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Compression method:");
                    ComboBox::from_id_source("Compression Method").selected_text(self.compression_settings.seven_zip_settings.compression_method.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.compression_method, "LZMA2".to_string(), "7ZMA2");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Dictionary size (MB):");
                    ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.dictionary_size, 1..=2048));
                });

                ui.horizontal(|ui| {
                    ui.label("Word size:");
                    ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.word_size, 5..=273));
                });

                ui.horizontal(|ui| {
                    ui.label("Solid block size:");
                    match self.compression_settings.seven_zip_settings.solid_block_size_unit.as_str() {
                        "g" => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.solid_block_size, 1..=64)),
                        _ => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.solid_block_size, 1..=65536))
                    };
                    ComboBox::from_id_source("Solid Block Size Unit").selected_text(self.compression_settings.seven_zip_settings.solid_block_size_unit.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.solid_block_size_unit, "m".to_string(), "MB");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.solid_block_size_unit, "g".to_string(), "GB");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("CPU Threads:");
                    let max_cpu_threads = std::thread::available_parallelism().unwrap().get() as u8;
                    ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.number_of_cpu_threads, 1..=max_cpu_threads));
                });

                ui.horizontal(|ui| {
                    ui.label("Split size:");
                    match self.compression_settings.seven_zip_settings.solid_block_size_unit.as_str() {
                        "g" => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.split_size, 0..=100)),
                        _ => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.split_size, 0..=65535))
                    };
                    ComboBox::from_id_source("Split Size Unit").selected_text(self.compression_settings.seven_zip_settings.solid_block_size_unit.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.split_size_unit, "m".to_string(), "MB");
                            ui.selectable_value(&mut self.compression_settings.seven_zip_settings.split_size_unit, "g".to_string(), "GB");
                        });
                });

                let memory = calculate_7zip_memory_usage(&self.compression_settings.seven_zip_settings);

                ui.horizontal(|ui| {
                    ui.label(format!("Memory usage for Compressing: {}", memory.0));
                    ui.label(format!("Memory usage for Decompressing: {}", memory.1));
                });
            }
            Archiver::WinRAR => {}
        }
    }
}

fn calculate_7zip_memory_usage(settings: &SevenZipSettings) -> (u64, u64) {
    if settings.compression_level == 0 {
        return (1, 1);
    }
    let dictionary_size: u64 = settings.dictionary_size as u64 * 1024;
    let mut size = 0;
    if settings.compression_level == 9 {
        size += 29 * 2_u64.pow(20);
    }

    let mut hs = dictionary_size - 1;
    // Find the highest bit set in hs
    let mut bit = 0;
    while hs > 0 {
        bit += 1;
        hs /= 2;
    }
    // Set all bits below the highest bit
    hs = 2_u64.pow(bit) - 1;
    // Set the lower 16 bits
    hs |= 0xFFFF;
    if hs > 2_u64.pow(24) {
        hs /= 2;
    }
    hs += 1;
    let mut size1 = hs * 4;
    size1 += dictionary_size * 4;
    if settings.compression_level >= 5 {
        size1 += dictionary_size * 4;
    }
    size1 += 2 * 2_u64.pow(20);

    let mut num_threads1 = 1;
    if settings.number_of_cpu_threads > 1 && settings.compression_level >= 5 {
        size1 += (2 + 4) * 2_u64.pow(20);
        num_threads1 = 2;
    }

    let num_block_threads = settings.number_of_cpu_threads / num_threads1;

    let mut chunk_size: u64 = dictionary_size * 2_u64.pow(2);
    chunk_size = std::cmp::max(chunk_size, 2_u64.pow(20));
    chunk_size = std::cmp::min(chunk_size, 2_u64.pow(28));
    chunk_size = std::cmp::max(chunk_size, dictionary_size);
    size1 += chunk_size * 2;

    size += size1 * num_block_threads as u64;

    (
        ((size) / 2_u64.pow(20)) as u64,
        (((dictionary_size) + 2_u64.pow(20)) / 2_u64.pow(20)) as u64,
    )
}

