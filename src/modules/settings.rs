use eframe::egui::{Context, TextEdit, Ui};
use crate::modules::depot_downloader::DepotDownloaderSettings;

#[derive(Default)]
pub struct SettingsUI {
    pub depot_downloader_settings: DepotDownloaderSettings,

}

impl SettingsUI {
    pub fn display(ctx: &Context, ui: &mut Ui, settings_ui: &mut SettingsUI) {
        settings_ui.display_depot_downloader_login(ui);

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
                    "Remember credentials (requires login with Depot Downloader at least once)");

    }
}