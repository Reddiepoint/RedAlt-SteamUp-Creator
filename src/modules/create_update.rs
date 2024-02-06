use std::cmp;
use std::ffi::OsStr;
use std::fmt::format;
use std::path::{Path, PathBuf};
use eframe::egui::{Context, ScrollArea, TextEdit, Ui};
use egui_file::FileDialog;
use crate::modules::app::{RedAltSteamUpdateCreator, TabBar};
use crate::modules::changes::Changes;
use crate::modules::depot_downloader::{DepotDownloaderSettings, download_changes};
use crate::modules::settings::SettingsUI;

#[derive(Default)]
pub struct CreateUpdateUI {
    open_file_dialog: Option<FileDialog>,
    changes_json_file: Option<PathBuf>,
    changes: Changes,
}

impl CreateUpdateUI {
    pub fn display(ctx: &Context, ui: &mut Ui, create_update_ui: &mut CreateUpdateUI, depot_downloader_settings: &DepotDownloaderSettings, tab_bar: &mut TabBar) {
        // Choose the JSON file
        create_update_ui.display_file_dialog(ctx, ui);
        // Parse and display the changes
        create_update_ui.display_changes(ui);
        if !create_update_ui.changes.depot.is_empty() {
            create_update_ui.display_download_stuff(ui, depot_downloader_settings, tab_bar);
        }
    }

    fn display_file_dialog(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match &self.changes_json_file {
                None => ui.label("Choose the JSON file:"),
                Some(path) => ui.label(format!("Using JSON file: {}", path.display())),
            };

            if ui.button("Open file").clicked() {
                // Show only files with the extension "json".
                let filter = Box::new({
                    let ext = Some(OsStr::new("json"));
                    move |path: &Path| -> bool { path.extension() == ext }
                });
                let mut dialog = FileDialog::open_file(self.changes_json_file.clone()).show_files_filter(filter);
                dialog.open();
                self.open_file_dialog = Some(dialog);
            }
        });

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.changes_json_file = Some(file.to_path_buf());
                }
            }
        }
    }

    fn display_changes(&mut self, ui: &mut Ui) {
        // Open file and deserialise it
        if let Some(file) = &self.changes_json_file {
            if let Ok(file) = std::fs::read_to_string(file) {
                // Get changes
                self.changes = serde_json::from_str::<Changes>(&file)
                    .unwrap_or_else(|error| { Changes::new_error(error.to_string()) });
                // Display changes
                let information = match !self.changes.depot.is_empty() {
                    true => format!("Creating update for {} ({} - {}) from Build {} to Build {}",
                                    self.changes.app, self.changes.depot, self.changes.manifest,
                                    self.changes.initial_build, self.changes.final_build),
                    false => format!("{}", self.changes.app)
                };
                ui.label(information);
                let lengths = [self.changes.added.len(), self.changes.removed.len(), self.changes.modified.len()];
                let num_columns = lengths.iter().filter(|&&x| x > 0).count();
                let max_length = lengths.iter().max();

                ScrollArea::both().max_height(ui.available_height() / 2.0).show(ui, |ui| {
                    ui.columns(num_columns, |columns| {
                        if !self.changes.added.is_empty() {
                            columns[0].heading("New files");
                            columns[0].add(TextEdit::multiline(&mut self.changes.added.join("\n").to_string())
                                .desired_rows(*max_length.unwrap()));
                            columns.rotate_left(1);
                        }

                        if !self.changes.removed.is_empty() {
                            columns[0].heading("Removed files");
                            columns[0].add(TextEdit::multiline(&mut self.changes.removed.join("\n").to_string())
                                .desired_rows(*max_length.unwrap()));
                            columns.rotate_left(1);
                        }

                        if !self.changes.modified.is_empty() {
                            columns[0].heading("Modified files");
                            columns[0].add(TextEdit::multiline(&mut self.changes.modified.join("\n").to_string())
                                .desired_rows(*max_length.unwrap()));
                        }
                    });
                });
            }
        }
    }

    fn display_download_stuff(&mut self, ui: &mut Ui, depot_downloader_settings: &DepotDownloaderSettings, mut tab_bar: &mut TabBar) {
        if depot_downloader_settings.remember_credentials {
            if ui.button("Download changes using last used credentials").clicked() {
                download_changes(&self.changes, depot_downloader_settings);
            }
        } else if depot_downloader_settings.username.is_empty() {
            if ui.button("Login").clicked() {
                *tab_bar = TabBar::Settings;
            }
        } else if ui.button(format!("Download changes as {}", depot_downloader_settings.username)).clicked() {
            download_changes(&self.changes, depot_downloader_settings);
        }
    }
}