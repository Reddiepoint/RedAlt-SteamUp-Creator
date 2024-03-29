use crate::modules::compression::{Archiver, CompressionSettings};
use crate::modules::compression_settings::{SevenZipSettings, WinRARSettings};
use crate::modules::depot_downloader::{DepotDownloaderSettings, EncryptionKey};
use eframe::egui::{ComboBox, Context, Slider, TextEdit, Ui};
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use aes_gcm::aead::{Aead, Nonce, OsRng};
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
            self.compression_settings = settings.compression_settings;

            // Decrypt username
            if let Ok(mut file) = std::fs::File::open("key.json") {
               self.depot_downloader_settings.encryption_key = serde_json::from_reader(&mut file).unwrap();

            };

            let key = xor_cipher(&self.depot_downloader_settings.encryption_key.encrypted_encryption_key);
            let nonce = Nonce::<Aes256Gcm>::from_slice(&self.depot_downloader_settings.username_nonce);
            if key.is_empty() || nonce.is_empty() {
                return;
            }

            let key = Key::<Aes256Gcm>::from_slice(&key);
            let cipher = Aes256Gcm::new(&key);

            let decryption = cipher.decrypt(&nonce, self.depot_downloader_settings.encrypted_username.as_ref());
            match decryption {
                Ok(decrypted_username) => {
                    self.depot_downloader_settings.username = String::from_utf8(decrypted_username).unwrap();
                }
                Err(_) => {}
            }
        }
    }

    fn set_settings(&mut self) {
        // let username = self.depot_downloader_settings.username.clone();
        if self.depot_downloader_settings.remember_credentials {
            let key = Aes256Gcm::generate_key(OsRng);
            let nonce = Aes256Gcm::generate_nonce(OsRng);
            let cipher = Aes256Gcm::new(&key);
            let decrypted_username = self.depot_downloader_settings.username.clone();
            let encrypted_username = cipher.encrypt(&nonce, decrypted_username.as_bytes());
            self.depot_downloader_settings.encryption_key.encrypted_encryption_key = xor_cipher(&key.as_slice().to_owned().try_into().unwrap());
            self.depot_downloader_settings.username_nonce = nonce.as_slice().to_owned().try_into().unwrap();
            self.depot_downloader_settings.encrypted_username = encrypted_username.unwrap();

            // self.depot_downloader_settings.username = String::new();
        } else {
            self.depot_downloader_settings.encryption_key.encrypted_encryption_key = [0; 32];
            self.depot_downloader_settings.username_nonce = [0; 12];
            self.depot_downloader_settings.encrypted_username = Vec::new();
        }
        let _ = std::fs::write("key.json", serde_json::to_string_pretty(&self.depot_downloader_settings.encryption_key).unwrap());
        let _ = std::fs::write("settings.json", serde_json::to_string_pretty(&self).unwrap());
        // self.depot_downloader_settings.username = username;
    }

    pub fn display(_ctx: &Context, ui: &mut Ui, settings_ui: &mut SettingsUI) {
        settings_ui.display_settings_buttons(ui);
        settings_ui.display_depot_downloader_settings(ui);
        ui.separator();
        settings_ui.display_compression_settings(ui);
        ui.separator();
        settings_ui.display_multiup_direct_settings(ui);
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
                    ui.selectable_value(&mut self.compression_settings.archiver, Archiver::SevenZip, "7-zip");
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
                self.display_7zip_settings(ui);
            }
            Archiver::WinRAR => {
                self.display_winrar_settings(ui);
            }
        }
    }

    fn display_7zip_settings(&mut self, ui: &mut Ui) {
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
                "g" => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.solid_block_size, 1..=100)),
                _ => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.solid_block_size, 1..=10000))
            };
            ComboBox::from_id_source("Solid Block Size Unit").selected_text(format!("{}B", self.compression_settings.seven_zip_settings.solid_block_size_unit.to_uppercase()))
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
            match self.compression_settings.seven_zip_settings.split_size_unit.as_str() {
                "g" => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.split_size, 0..=100)),
                _ => ui.add(Slider::new(&mut self.compression_settings.seven_zip_settings.split_size, 0..=10000))
            };
            ComboBox::from_id_source("Split Size Unit").selected_text(format!("{}B", self.compression_settings.seven_zip_settings.split_size_unit.to_uppercase()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.compression_settings.seven_zip_settings.split_size_unit, "m".to_string(), "MB");
                    ui.selectable_value(&mut self.compression_settings.seven_zip_settings.split_size_unit, "g".to_string(), "GB");
                });
        });

        let memory = calculate_7zip_memory_usage(&self.compression_settings.seven_zip_settings);

        ui.horizontal(|ui| {
            ui.label(format!("Memory usage for Compressing: {} MB.", memory.0));
            ui.label(format!("Memory usage for Decompressing: {} MB.", memory.1));
        });
    }

    fn display_winrar_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Password");
            ui.text_edit_singleline(&mut self.compression_settings.win_rar_settings.password);
        });

        ui.horizontal(|ui| {
            ui.label("Archive format:");
            ComboBox::from_id_source("Format").selected_text(self.compression_settings.win_rar_settings.archive_format.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.archive_format, "rar".to_string(), "RAR");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Compression method:");
            ComboBox::from_id_source("Compression Method").selected_text(format!("{}", self.compression_settings.win_rar_settings.compression_level))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 0, "0 - Store");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 1, "1 - Fastest");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 2, "2 - Fast");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 3, "3 - Normal");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 4, "4 - Good");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.compression_level, 5, "5 - Best");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Dictionary size (MB):");
            ui.add(Slider::new(&mut self.compression_settings.win_rar_settings.dictionary_size, 1..=1024));
            // Round to nearest power of 2
            self.compression_settings.win_rar_settings.dictionary_size = self.compression_settings.win_rar_settings.dictionary_size.next_power_of_two();
        });

        ui.checkbox(&mut self.compression_settings.win_rar_settings.solid, "Solid");

        ui.horizontal(|ui| {
            ui.label("CPU Threads:");
            let max_cpu_threads = std::thread::available_parallelism().unwrap().get() as u8;
            ui.add(Slider::new(&mut self.compression_settings.win_rar_settings.number_of_cpu_threads, 1..=max_cpu_threads));
        });

        ui.horizontal(|ui| {
            ui.label("Split size:");
            match self.compression_settings.win_rar_settings.split_size_unit.as_str() {
                "g" => ui.add(Slider::new(&mut self.compression_settings.win_rar_settings.split_size, 0..=100)),
                _ => ui.add(Slider::new(&mut self.compression_settings.win_rar_settings.split_size, 0..=10000))
            };
            ComboBox::from_id_source("Split Size Unit").selected_text(format!("{}B", self.compression_settings.win_rar_settings.split_size_unit.to_uppercase()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.split_size_unit, "m".to_string(), "MB");
                    ui.selectable_value(&mut self.compression_settings.win_rar_settings.split_size_unit, "g".to_string(), "GB");
                });
        });

        let memory = calculate_winrar_memory_usage(&self.compression_settings.win_rar_settings);

        ui.horizontal(|ui| {
            ui.label(format!("Memory usage for Compressing: {} MB.", memory.0));
            ui.label(format!("Memory usage for Decompressing: {} MB.", memory.1));
        });
    }

    fn display_multiup_direct_settings(&mut self, ui: &mut Ui) {
        ui.heading("MultiUp Direct Settings");
        ui.horizontal(|ui| {
            match &self.compression_settings.multiup_direct_path {
                Some(path) => ui.label(format!("Using executable: {}", path.display())),
                None => ui.label("Choose the executable:"),
            };

            if ui.button("Open file").clicked() {
                // Show only files with the extension "json".
                let filter = Box::new({
                    let ext = Some(OsStr::new("exe"));
                    move |path: &Path| -> bool { path.extension() == ext }
                });
                let mut dialog = FileDialog::open_file(self.compression_settings.multiup_direct_path.clone()).show_files_filter(filter);
                dialog.open();
                self.compression_settings.multiup_direct_file_dialog = Some(dialog);
            }
        });

        ui.hyperlink_to("Download MultiUp Direct", "https://cs.rin.ru/forum/viewtopic.php?p=2822500");

        if let Some(dialog) = &mut self.compression_settings.multiup_direct_file_dialog {
            if dialog.show(ui.ctx()).selected() {
                if let Some(file) = dialog.path() {
                    self.compression_settings.multiup_direct_path = Some(file.to_path_buf());
                }
            }
        }
    }
}

fn xor_cipher(key: &[u8; 32]) -> [u8; 32] {
    let secret_key = "red-alt-steam-up-creator-xor-key".as_bytes();
    let mut output = [0; 32];
    for i in 0..32 {
        output[i] = key[i] ^ secret_key[i];
    }
    output
}

fn calculate_7zip_memory_usage(settings: &SevenZipSettings) -> (u128, u128) {
    if settings.compression_level == 0 {
        return (1, 1);
    }

    let bytes_ratio = 1024 * 1024;
    // Convert the dictionary size to bytes
    let dictionary_size = settings.dictionary_size as u128 * bytes_ratio;

    let mut size = 0;

    if settings.archive_format == "7z" && settings.compression_level == 9 {
        size += 29 * bytes_ratio;
    }

    let mut hs = dictionary_size - 1;
    hs |= hs >> 1;
    hs |= hs >> 2;
    hs |= hs >> 4;
    hs |= hs >> 8;
    hs >>= 1;
    hs |= 0xFFFF;
    if hs > (1 << 24) {
        hs >>= 1;
    }
    hs += 1;

    let mut size_1 = hs * 4;
    size_1 += dictionary_size * 4;
    if settings.compression_level >= 5 {
        size_1 += dictionary_size * 4;
    }
    size_1 += 2 * bytes_ratio;

    let mut num_threads_1 = 1;
    if settings.number_of_cpu_threads > 1 && settings.compression_level >= 5 {
        size_1 += (2 * bytes_ratio) + (4 * bytes_ratio);
        num_threads_1 = 2;
    }

    let num_block_threads = settings.number_of_cpu_threads / num_threads_1;

    if num_block_threads == 1 {
        size_1 += (dictionary_size * 3) / 2;
    } else {
        let mut chunk_size = dictionary_size * 4;
        chunk_size = chunk_size.max(bytes_ratio);
        chunk_size = chunk_size.min(256 * bytes_ratio);
        chunk_size = chunk_size.max(dictionary_size);
        size_1 += chunk_size * 2;
    }

    // calculate the solid block size
    let mut solid_block_size = dictionary_size;
    if settings.compression_level == 9 {
        solid_block_size *= num_block_threads as u128;
    }

    size += size_1 * num_block_threads as u128 + solid_block_size;

    // Return the size for compression and decompression in MB
    (
        (size + bytes_ratio - 1) / bytes_ratio,
        (dictionary_size + (2 * bytes_ratio) + bytes_ratio - 1) / bytes_ratio,
    )
}

fn calculate_winrar_memory_usage(settings: &WinRARSettings) -> (u16, u16) {
    (settings.dictionary_size * 6, settings.dictionary_size)
}
