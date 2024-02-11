use eframe::egui::{Context, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use crate::modules::depot_downloader::DepotDownloaderSettings;

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
        settings_ui.display_depot_downloader_login(ui);
        // let _ = std::fs::write("last_user.txt", settings_ui.depot_downloader_settings.username.clone());
        settings_ui.set_settings();
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
                    "Remember password (Requires login with Depot Downloader at least once. \
                    Subsequent logins require the username only.)");
    }
}

