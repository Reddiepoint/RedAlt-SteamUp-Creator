use eframe::egui::{ComboBox, Context, Slider, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use crate::modules::depot_downloader::{DepotDownloaderSettings, OSType};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SettingsUI {
    pub depot_downloader_settings: DepotDownloaderSettings,
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
        let mut settings = self.clone();
        if !self.depot_downloader_settings.remember_credentials {
            settings.depot_downloader_settings.username = "".to_string();
        }
        let _ = std::fs::write("settings.json", serde_json::to_string_pretty(&settings).unwrap());
    }

    pub fn display(ctx: &Context, ui: &mut Ui, settings_ui: &mut SettingsUI) {
        settings_ui.display_settings_buttons(ui);
        settings_ui.display_depot_downloader_login(ui);
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
    fn display_depot_downloader_login(&mut self, ui: &mut Ui) {
        ui.heading("Steam Depot Downloader Login");
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
            ui.label("OS:");
            ComboBox::from_id_source("OS").selected_text(format!("{}", self.depot_downloader_settings.os))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Windows, "Windows");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Linux, "Linux");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Mac, "Mac");
                    ui.selectable_value(&mut self.depot_downloader_settings.os, OSType::Current, "Current OS");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Max number of server connections");
            ui.add(Slider::new(&mut self.depot_downloader_settings.max_servers, 1..=32));
        });

        ui.horizontal(|ui| {
            ui.label("Max number of concurrent chunks downloaded");
            ui.add(Slider::new(&mut self.depot_downloader_settings.max_downloads, 1..=32));
        });
    }
}

