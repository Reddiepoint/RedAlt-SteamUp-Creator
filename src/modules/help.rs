use std::env::current_dir;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui::{Button, Context, Window};
use self_update::update::Release;
use self_update::version::{bump_is_compatible, bump_is_greater};

#[derive(Default)]
pub enum UpdateStatus {
    #[default]
    Unchecked,
    Checking,
    Outdated,
    Updated,
    Error(String),
}

enum AppType {
    Creator,
    Installer
}

struct Channels {
    pub release_sender: Sender<Result<((Release, String, bool), (Release, String, bool)), String>>,
    pub release_receiver: Receiver<Result<((Release, String, bool), (Release, String, bool)), String>>,
    pub update_status_sender: Sender<Result<AppType, (AppType, String)>>,
    pub update_status_receiver: Receiver<Result<AppType, (AppType, String)>>,

}

impl Default for Channels {
    fn default() -> Self {
        let (release_sender, release_receiver) = crossbeam_channel::bounded(1);
        let (update_status_sender, update_status_receiver) = crossbeam_channel::bounded(3);
        Self {
            release_sender,
            release_receiver,
            update_status_sender,
            update_status_receiver,
        }
    }
}

#[derive(Default)]
struct LatestVersions {
    creator: (Release, String, bool),
    installer: (Release, String, bool),
    // depot_downloader: self_update::update::Release,
}

#[derive(Default)]
struct AllUpdateStatus {
    checked: UpdateStatus,
    creator: UpdateStatus,
    installer: UpdateStatus,
}

#[derive(Default)]
pub struct HelpUI {
    pub show_help: bool,
    pub show_update: bool,
    channels: Channels,
    update_status: AllUpdateStatus,
    latest_versions: LatestVersions,
    updating: (bool, bool),
    creator_status: String,
    installer_status: String,
    pub link_to_latest_version: String,
}


// const HOMEPAGE: &str = "https://cs.rin.ru/forum/viewtopic.php?f=14&p=2822500#p2822500";
// const DOCUMENTATION: &str = "https://reddiepoint.github.io/MultiUp-Direct-Documentation/";

impl HelpUI {
    pub fn display(&mut self, ctx: &Context) {}
    // pub fn show_help(ctx: &Context, help_ui: &mut HelpUI) {
    //     Window::new("Help").open(&mut help_ui.show_help).show(ctx, |ui| ScrollArea::vertical().min_scrolled_height(ui.available_height()).id_source("Help").show(ui, |ui| {
    //         ui.horizontal(|ui| {
    //             ui.hyperlink_to("Tips & Tricks and Extra Information", DOCUMENTATION);
    //             ui.label("|");
    //             ui.hyperlink_to("Homepage", HOMEPAGE);
    //         });
    //
    //         ui.heading("Extract");
    //         ui.label("Extracts direct links from MultiUp links.\n\n\
    //         Link detection is quite robust, meaning you can paste in any page with links as well as HTML containing links. \
    //         Duplicate links will be filtered out, excluding links in projects.\n\n\
    //         If you want the validity of the hosts to be checked by MultiUp, enable \"Recheck link validity,\" \
    //         otherwise, the original values from the site will be used. However, generation times may take much longer if this is enabled.\n\n\
    //         You can select direct links by using combinations of CTRL and SHIFT and clicking and search for file names.");
    //
    //         ui.separator();
    //
    //         ui.heading("Debrid");
    //         ui.label("Unlocks links using a Debrid service.\n\n\
    //         Currently supports AllDebrid and RealDebrid.\n\
    //         To read the keys from a file, create \"api_key.json\" in the same directory as this app with the following structure:");
    //         let mut json_example = "\
    //         {\n\
    //             \t\"all_debrid\": \"YOUR_ALLDEBRID_API_KEY\",\n\
    //             \t\"real_debrid\": \"YOUR_REALDEBRID_API_KEY\"\n\
    //         }";
    //         ui.code_editor(&mut json_example);
    //         ui.label("You can choose to omit any field here (i.e. only have all_debrid or real_debrid) \
    //         if you do not have an API key for the service.");
    //
    //         ui.separator();
    //
    //         ui.heading("Upload");
    //         ui.label("Uploads content to MultiUp.\n\n\
    //         Remote uploaded with data streaming enabled allows for better support of different sites, including Debrid services.\
    //         Since this is an experimental feature, be careful when uploading large files.\n\
    //         Data streaming essentially downloads and uploads chunks of data, as if the file was downloaded \
    //         to disk and then uploaded to MultiUp. However, in this case, the data is not written to disk.");
    //     }));
    // }

    pub fn show_update_window(&mut self, ctx: &Context) {
        Window::new("Updates").open(&mut self.show_update).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading({
                    match &self.update_status.checked {
                        UpdateStatus::Unchecked | UpdateStatus::Checking => "Checking for updates...".to_string(),
                        UpdateStatus::Outdated => "There is an update available!".to_string(),
                        UpdateStatus::Updated => "You are up-to-date!".to_string(),
                        UpdateStatus::Error(error) => format!("Update failed: {}", error)
                    }
                });

                if let UpdateStatus::Checking = self.update_status.checked {
                    ui.spinner();
                };
            });


            // ui.hyperlink_to("Homepage", HOMEPAGE);


            match self.update_status.checked {
                UpdateStatus::Unchecked => {
                    let release_sender = self.channels.release_sender.clone();
                    thread::spawn(move || {
                        match HelpUI::check_for_updates() {
                            Ok(releases) => {
                                let _ = release_sender.send(Ok(releases));
                            },
                            Err(error) => {
                                let _ = release_sender.send(Err(error.to_string()));
                            }
                        };
                    });
                    self.update_status.checked = UpdateStatus::Checking;
                }

                UpdateStatus::Checking => {
                    if let Ok(update) = self.channels.release_receiver.try_recv() {
                        match update {
                            Ok((creator_release, installer_release)) => {
                                self.latest_versions.creator = creator_release;
                                self.latest_versions.installer = installer_release;

                                if self.latest_versions.creator.2 || self.latest_versions.installer.2 {
                                    self.update_status.checked = UpdateStatus::Outdated;
                                } else {
                                    self.update_status.checked = UpdateStatus::Updated;
                                }
                            }
                            Err(error) => {
                                self.update_status.checked = UpdateStatus::Error(error);
                            }
                        }
                    }
                },
                _ => {}
            };


            if let Ok(status) = self.channels.update_status_receiver.try_recv() {
                match status {
                    Ok(app) => {
                        match app {
                            AppType::Creator => {
                                self.updating.0 = false;
                                self.creator_status = "success".to_string()
                            },
                            AppType::Installer => {
                                self.updating.1 = false;
                                self.installer_status = "success".to_string()
                            },
                        }
                    }
                    Err((app, error)) => {
                        match app {
                            AppType::Creator => {
                                self.updating.0 = false;
                                self.creator_status = error
                            },
                            AppType::Installer => {
                                self.updating.1 = false;
                                self.installer_status = error
                            },
                        }
                    }
                }
            }

            ui.separator();

            ui.heading("RedAlt Steam Update Creator");

            match self.latest_versions.creator.2 {
                true => {
                    ui.label(format!("Update available from v{} -> v{}", self.latest_versions.creator.1, self.latest_versions.creator.0.version));
                    if ui.add_enabled(!self.updating.0, Button::new("Update")).clicked() {
                        self.updating.0 = true;
                        let update_status_sender = self.channels.update_status_sender.clone();
                        let release_sender = self.channels.release_sender.clone();
                        thread::spawn(move || {
                            match HelpUI::update(AppType::Creator) {
                                Ok(app) => {
                                    let _ = update_status_sender.send(Ok(app));
                                },
                                Err(error) => {
                                    let _ = update_status_sender.send(Err((AppType::Creator, error.to_string())));
                                }
                            };

                            match HelpUI::check_for_updates() {
                                Ok(releases) => {
                                    let _ = release_sender.send(Ok(releases));
                                },
                                Err(error) => {
                                    let _ = release_sender.send(Err(error.to_string()));
                                }
                            };
                        });
                    }

                    if !self.creator_status.is_empty() {
                        if self.creator_status == "success" {
                            ui.label("Please restart the application to use the latest version!");
                        } else {
                            ui.label(format!("Error updating creator: {}", self.creator_status));
                        }
                    }
                }
                false => {
                    ui.label("No update available.");
                }
            }

            ui.separator();

            ui.heading("RedAlt Steam Update Installer");

            match self.latest_versions.installer.2 {
                true => {
                    ui.label(format!("Update available from v{} -> v{}", self.latest_versions.installer.1, self.latest_versions.installer.0.version));
                    if ui.add_enabled(!self.updating.1, Button::new("Update")).clicked() {
                        self.updating.1 = true;
                        let update_status_sender = self.channels.update_status_sender.clone();
                        thread::spawn(move || {
                            match HelpUI::update(AppType::Installer) {
                                Ok(app) => {
                                    let _ = update_status_sender.send(Ok(app));
                                },
                                Err(error) => {
                                    let _ = update_status_sender.send(Err((AppType::Installer, error.to_string())));
                                }
                            };
                        });
                    }

                    if !self.installer_status.is_empty() {
                        if self.installer_status == "success" {
                            ui.label("Updated RedAlt Steam Update Installer to the latest version!");
                        } else {
                            ui.label(format!("Error updating installer: {}", self.installer_status));
                        }
                    }
                }
                false => {
                    ui.label("No update available.");
                }
            }
        });
    }

    fn check_for_updates() -> Result<((Release, String, bool), (Release, String, bool)), Box<dyn std::error::Error>> {
        let creator_current_version = env!("CARGO_PKG_VERSION").to_string();
        let creator_update = self_update::backends::github::Update::configure()
            .repo_owner("Reddiepoint")
            .repo_name("RedAlt-Steam-Update-Creator")
            .target("")
            .bin_name("RedAlt-Steam-Update-Creator")
            .current_version(&creator_current_version)
            .build()?
            .get_latest_release()?;

        let mut command = Command::new("./RedAlt-Steam-Update-Installer.exe");
        command
            .creation_flags(0x08000000)
            .stdout(Stdio::piped())
            .arg("--version");
        let installer_current_version = match command.spawn() {
            Ok(child) => {
                let child = child.wait_with_output().unwrap();
                String::from_utf8(child.stdout).unwrap().trim().to_string()
            }
            Err(_) => "0.0.0".to_string(),
        };

        let installer_update = self_update::backends::github::Update::configure()
            .repo_owner("Reddiepoint")
            .repo_name("RedAlt-Steam-Update-Installer")
            .target("")
            .bin_name("RedAlt-Steam-Update-Installer")
            .current_version(&installer_current_version)
            .build()?
            .get_latest_release()?;

        let is_creator_update_greater = bump_is_greater(&creator_current_version, &creator_update.version).unwrap();
        let is_installer_update_greater = bump_is_greater(&installer_current_version, &installer_update.version).unwrap();

        Ok((
            (creator_update, creator_current_version, is_creator_update_greater),
            (installer_update, installer_current_version, is_installer_update_greater)
        ))
    }

    fn update(app: AppType) -> Result<AppType, Box<dyn std::error::Error>> {
        match app {
            AppType::Creator => {
                self_update::backends::github::Update::configure()
                    .repo_owner("Reddiepoint")
                    .repo_name("RedAlt-Steam-Update-Creator")
                    .target("")
                    .bin_name("RedAlt-Steam-Update-Creator")
                    .show_download_progress(false)
                    .show_output(false)
                    .no_confirm(true)
                    .current_version(env!("CARGO_PKG_VERSION"))
                    .build()?
                    .update()?;
                Ok(app)
            },
            AppType::Installer => {
                let latest_release = self_update::backends::github::Update::configure()
                    .repo_owner("Reddiepoint")
                    .repo_name("RedAlt-Steam-Update-Installer")
                    .target("")
                    .bin_name("RedAlt-Steam-Update-Installer")
                    .current_version("0.0.0")
                    .build()?
                    .get_latest_release()?;

                let windows_build = latest_release.asset_for("", None).unwrap();
                let linux_build = latest_release.asset_for("amd64", None).unwrap();
                let mac_build = latest_release.asset_for("darwin", None).unwrap();

                let builds = [(windows_build, ""), (linux_build, "amd64"), (mac_build, "darwin")];
                let temp_folder = tempfile::Builder::new()
                    // .prefix("redalt")
                    .tempdir_in(current_dir()?)?;

                for (build, platform) in builds {
                    let temp_zip_path = temp_folder.path().join(&build.name);
                    let temp_zip = std::fs::File::create(&temp_zip_path)?;
                    self_update::Download::from_url(&build.download_url)
                        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
                        .download_to(&temp_zip)?;
                    let name = if platform.is_empty() {
                        "RedAlt-Steam-Update-Installer.exe".to_string()
                    } else {
                        format!("RedAlt-Steam-Update-Installer_{}", platform)
                    };
                    let bin_name = std::path::PathBuf::from(&name);
                    self_update::Extract::from_source(&temp_zip_path)
                        .archive(self_update::ArchiveKind::Zip)
                        .extract_file(temp_folder.path(), &bin_name)?;

                    let new_exe = temp_folder.path().join(bin_name);
                    std::fs::write(name, std::fs::read(new_exe)?)?;
                }

                Ok(app)
            }
        }
    }
}
