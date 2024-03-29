use std::env::current_dir;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{Button, ComboBox, Context, ScrollArea, TextEdit, Ui, Window};
use egui_file::FileDialog;
use crate::modules::app::TabBar;
use crate::modules::changes::Changes;
use crate::modules::compression::{Archiver, CompressionSettings};
use crate::modules::depot_downloader::{DepotDownloaderSettings, download_changes, download_manifest};


pub struct CreateUpdateChannels {
    input_window_opened_sender: Sender<bool>,
    input_window_opened_receiver: Receiver<bool>,
    input_sender: Sender<String>,
    input_receiver: Receiver<String>,
    output_sender: Sender<String>,
    output_receiver: Receiver<String>,
    depot_downloader_path_sender: Sender<std::io::Result<PathBuf>>,
    depot_downloader_path_receiver: Receiver<std::io::Result<PathBuf>>,
    compression_status_sender: Sender<std::io::Result<()>>,
    compression_status_receiver: Receiver<std::io::Result<()>>,
}

impl Default for CreateUpdateChannels {
    fn default() -> Self {
        let (input_window_opened_sender, input_window_opened_receiver) = crossbeam_channel::bounded(1);
        let (input_sender, input_receiver) = crossbeam_channel::bounded(1);
        let (output_sender, output_receiver) = crossbeam_channel::unbounded();
        let (depot_downloader_path_sender, depot_downloader_path_receiver) = crossbeam_channel::bounded(1);
        let (compression_status_sender, compression_status_receiver) = crossbeam_channel::bounded(1);
        Self {
            input_window_opened_sender,
            input_window_opened_receiver,
            input_sender,
            input_receiver,
            output_sender,
            output_receiver,
            depot_downloader_path_sender,
            depot_downloader_path_receiver,
            compression_status_sender,
            compression_status_receiver,
        }
    }
}

#[derive(PartialEq)]
enum TargetOS {
    Windows,
    Linux,
    Mac
}

impl Display for TargetOS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TargetOS::Windows => "Windows",
            TargetOS::Linux => "Linux",
            TargetOS::Mac => "Mac"
        })
    }
}

pub struct CreateUpdateUI {
    channels: CreateUpdateChannels,
    open_file_dialog: Option<FileDialog>,
    changes_json_file: Option<PathBuf>,
    changes: Changes,
    target_os: TargetOS,
    compress_files: bool,
    stdout: String,
    child_process_running: bool,
}

impl Default for CreateUpdateUI {
    fn default() -> Self {
        Self {
            channels: CreateUpdateChannels::default(),
            open_file_dialog: None,
            changes_json_file: None,
            changes: Changes::default(),
            target_os: TargetOS::Windows,
            compress_files: true,
            stdout: String::new(),
            child_process_running: false,
        }
    }
}

impl CreateUpdateUI {
    pub fn display(ctx: &Context, ui: &mut Ui, create_update_ui: &mut CreateUpdateUI,
                   depot_downloader_settings: &mut DepotDownloaderSettings, compression_settings: &mut CompressionSettings,
                   tab_bar: &mut TabBar) {
        // Choose the JSON file
        create_update_ui.display_file_dialog(ctx, ui);
        // Parse and display the changes
        create_update_ui.display_changes(ui);
        if !create_update_ui.changes.depot.is_empty() {
            create_update_ui.display_download_stuff(ui, depot_downloader_settings, compression_settings, tab_bar);
            create_update_ui.display_depot_downloader_input_window(ui, depot_downloader_settings);
            ui.separator();
            create_update_ui.display_stdout(ui);
            create_update_ui.multiup_direct_button(ui, compression_settings);
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
                    true => format!("Creating update for {} ({}) (Depot {} - {}) from Build {} to Build {}",
                                    self.changes.name, self.changes.app, self.changes.depot, self.changes.manifest,
                                    self.changes.initial_build, self.changes.final_build),
                    false => self.changes.app.to_string()
                };
                ui.label(information);
                let lengths = [self.changes.added.len(), self.changes.removed.len(), self.changes.modified.len()];
                let num_columns = lengths.iter().filter(|&&x| x > 0).count();
                let max_length = lengths.iter().max();

                ScrollArea::both().id_source("Changes").max_height(ui.available_height() / 3.0).show(ui, |ui| {
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

    fn display_download_stuff(&mut self, ui: &mut Ui, depot_downloader_settings: &mut DepotDownloaderSettings,
                              compression_settings: &mut CompressionSettings, tab_bar: &mut TabBar) {
        ui.horizontal(|ui| {
            ui.label("Target OS: ");
            ComboBox::from_id_source("Target OS").selected_text(format!("{}", self.target_os))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.target_os, TargetOS::Windows, "Windows");
                    ui.selectable_value(&mut self.target_os, TargetOS::Linux, "Linux");
                    ui.selectable_value(&mut self.target_os, TargetOS::Mac, "Mac");
                });
        });

        ui.checkbox(&mut depot_downloader_settings.download_entire_depot, "Ignore changes and download entire depot");
        ui.checkbox(&mut depot_downloader_settings.download_manifest, "Download manifest");
        ui.checkbox(&mut self.compress_files, "Compress files after downloading");

        if !depot_downloader_settings.username.is_empty() && (!depot_downloader_settings.password.is_empty() || depot_downloader_settings.remember_credentials) {
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.child_process_running, Button::new(format!("Download changes as {}", depot_downloader_settings.username))).clicked() {
                    let changes = self.changes.clone();
                    let depot_downloader_settings = depot_downloader_settings.clone();
                    let sender = self.channels.input_window_opened_sender.clone();
                    let receiver = self.channels.input_receiver.clone();
                    let path_sender = self.channels.depot_downloader_path_sender.clone();
                    let stdio_sender = self.channels.output_sender.clone();
                    self.child_process_running = true;
                    thread::spawn(move || {
                        let status = download_changes(&changes, &depot_downloader_settings, sender, receiver, stdio_sender);
                        let _ = path_sender.send(status);
                    });
                }

                if self.child_process_running {
                    ui.spinner();
                }
            });
        } else if ui.button("Login").clicked() {
            *tab_bar = TabBar::Settings;
        }

        if let Ok(status) = self.channels.depot_downloader_path_receiver.try_recv() {
            match status {
                Ok(download_path) => {
                    let _ = self.channels.output_sender.send("Depot Downloader exited.\n".to_string());
                    compression_settings.download_path = download_path.clone();
                    // Copy JSON changes file to download path
                    if !depot_downloader_settings.download_entire_depot {
                        let installer_path = download_path.join(".RedAlt-SteamUp-Installer");
                        let _ = create_dir(&installer_path);
                        if let Some(file) = &self.changes_json_file {
                            let changes_path = installer_path.join(file.file_name().unwrap());
                            let _ = std::fs::copy(file, changes_path).unwrap();
                        }

                        if depot_downloader_settings.download_manifest {
                            let _ = std::fs::rename(
                                download_path.join(format!("manifest_{}_{}.txt", self.changes.depot, self.changes.manifest)),
                                installer_path.join(format!("manifest_{}_{}.txt", self.changes.depot, self.changes.manifest))
                            );
                        }
                        match self.target_os {
                            TargetOS::Windows => {
                                let installer_executable = "RedAlt-SteamUp-Installer.exe";
                                let _ = std::fs::copy(current_dir().unwrap().join(installer_executable), installer_path.join(installer_executable));
                            }
                            TargetOS::Linux => {
                                let installer_executable = "RedAlt-SteamUp-Installer_amd64";
                                let _ = std::fs::copy(current_dir().unwrap().join(installer_executable), installer_path.join(installer_executable));
                            }
                            TargetOS::Mac => {
                                let installer_executable = "RedAlt-SteamUp-Installer_darwin";
                                let _ = std::fs::copy(current_dir().unwrap().join(installer_executable), installer_path.join(installer_executable));
                            }
                        }
                    }

                    if self.compress_files {
                        let archiver = compression_settings.archiver.clone();
                        let download_path = compression_settings.download_path.clone();
                        let seven_zip_settings = compression_settings.seven_zip_settings.clone();
                        let win_rar_settings = compression_settings.win_rar_settings.clone();
                        let input_window_opened_sender = self.channels.input_window_opened_sender.clone();
                        let input_receiver = self.channels.input_receiver.clone();
                        let output_sender = self.channels.output_sender.clone();
                        let status_sender = self.channels.compression_status_sender.clone();
                        thread::spawn(move || {
                            let status = match archiver {
                                Archiver::SevenZip => seven_zip_settings.compress(download_path.clone(), input_window_opened_sender, input_receiver, output_sender),
                                Archiver::WinRAR => win_rar_settings.compress(download_path.clone(), input_window_opened_sender, input_receiver, output_sender),
                            };

                            let _ = status_sender.send(status);
                        });
                    } else {
                        let _ = self.channels.compression_status_sender.send(Ok(()));
                    }
                }
                Err(error) => {
                    let _ = self.channels.output_sender.send(format!("Depot Downloader exited unsuccessfully: {}.\n", error));

                    self.child_process_running = false;
                }
            }
        }

        if let Ok(status) = self.channels.compression_status_receiver.try_recv() {
            match status {
                Ok(_) => {
                    if self.compress_files {
                        let _ = self.channels.output_sender.send("\nFinished compressing files.\n".to_string());
                    }
                }
                Err(error) => {
                    let _ = self.channels.output_sender.send(format!("\nFailed to compress files: {}.\n", error));
                }
            }
            self.child_process_running = false;
        }
    }

    fn display_depot_downloader_input_window(&mut self, ui: &mut Ui, depot_downloader_settings: &mut DepotDownloaderSettings) {
        if let Ok(open) = self.channels.input_window_opened_receiver.try_recv() {
            depot_downloader_settings.depot_downloader_input_window_opened = open;
        }

        let mut open = depot_downloader_settings.depot_downloader_input_window_opened;
        Window::new("Depot Downloader Input").open(&mut depot_downloader_settings.depot_downloader_input_window_opened)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Enter input:");
                    ui.text_edit_singleline(&mut depot_downloader_settings.input);
                });

                open = if ui.button("Submit").clicked() {
                    self.channels.input_sender.send(depot_downloader_settings.input.clone()).unwrap();
                    false
                } else {
                    open
                };
            });

        depot_downloader_settings.depot_downloader_input_window_opened = open;
    }

    fn display_stdout(&mut self, ui: &mut Ui) {
        let mut output = self.stdout.clone();
        ScrollArea::vertical().id_source("Standard Output").max_height(ui.available_height() * 2.0 / 3.0).show(ui, |ui| {
            ui.add(TextEdit::multiline(&mut output).desired_width(ui.available_width()).cursor_at_end(true));
            while let Ok(output) = self.channels.output_receiver.try_recv() {
                self.stdout += &output;
                ui.scroll_to_cursor(None);
                ui.ctx().request_repaint();
            }
        });
    }

    fn multiup_direct_button(&mut self, ui: &mut Ui, compression_settings: &mut CompressionSettings) {
        // Check if there is an executable in the current directory
        if let Some(path) = &compression_settings.multiup_direct_path {
            if ui.button("Upload with MultiUp Direct").clicked() {
                let download_path = compression_settings.download_path.clone();
                let path = path.clone();
                thread::spawn(move || {
                    let mut command = Command::new(path);
                    command.args(["--upload", "disk_upload"]);
                    for entry in current_dir().unwrap().join("Completed").read_dir().unwrap().flatten() {
                        if entry.file_name().to_str().unwrap().contains(download_path.file_name().unwrap().to_str().unwrap()) {
                            if entry.file_type().unwrap().is_file() {
                                command.arg(entry.path());
                            } else if entry.file_type().unwrap().is_dir() {
                                for file in entry.path().read_dir().unwrap().flatten() {
                                    command.arg(file.path());
                                }
                            }
                        }
                    }
                    println!("{:?}", command);
                    let mut child = command.spawn().unwrap();
                    let _ = child.wait();
                });
            }
        }
    }
}