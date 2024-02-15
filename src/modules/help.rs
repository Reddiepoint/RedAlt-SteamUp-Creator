use std::env::current_dir;
use std::error::Error;
use std::thread;
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{Context, ScrollArea, Window};

#[derive(Default)]
pub enum UpdateStatus {
    #[default]
    Unchecked,
    Checking,
    Outdated,
    Updated,
}

struct Channels {
    pub update_sender: Sender<(String, Vec<String>)>,
    pub update_receiver: Receiver<(String, Vec<String>)>
}

impl Default for Channels {
    fn default() -> Self {
        let (update_sender, update_receiver) = crossbeam_channel::unbounded();
        Self {
            update_sender,
            update_receiver,
        }
    }
}
#[derive(Default)]
pub struct HelpUI {
    pub show_help: bool,
    pub show_update: bool,
    channels: Channels,
    pub update_status: UpdateStatus,
    pub latest_changelog: Vec<String>,
    pub latest_version: String,
    pub link_to_latest_version: String,
}


// const HOMEPAGE: &str = "https://cs.rin.ru/forum/viewtopic.php?f=14&p=2822500#p2822500";
// const DOCUMENTATION: &str = "https://reddiepoint.github.io/MultiUp-Direct-Documentation/";

impl HelpUI {
    pub fn display(&mut self, ctx: &Context) {

    }
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

    pub fn show_update_window(&mut self, ctx: &Context, ) {
        Window::new("Updates").open(&mut self.show_update).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading({
                    match self.update_status {
                        UpdateStatus::Unchecked => "Checking for updates...",
                        UpdateStatus::Checking => "Checking for updates...",
                        UpdateStatus::Outdated => "There is an update available!",
                        UpdateStatus::Updated => "You are up-to-date!",
                    }
                });

                if let UpdateStatus::Checking = self.update_status {
                    ui.spinner();
                };
            });


            // ui.hyperlink_to("Homepage", HOMEPAGE);


            match self.update_status {
                UpdateStatus::Unchecked => {
                    let update_sender = self.channels.update_sender.clone();
                    thread::spawn(move || {
                        match HelpUI::update() {
                            Ok(_) => {}
                            Err(error) => {
                                eprintln!("Error {}", error);
                            }
                        };
                        let _ = update_sender.send(("a".to_string(), vec!["a".to_string()])).unwrap();
                    });
                    self.update_status = UpdateStatus::Checking;
                }
                UpdateStatus::Outdated => {}
                UpdateStatus::Updated => {}
                UpdateStatus::Checking => {
                    if let Ok((latest_version, changelog)) = self.channels.update_receiver.clone().try_recv() {
                        self.update_status = UpdateStatus::Updated;
                    }
                }
            };
        });
    }

    pub fn update() -> Result<(), Box<dyn std::error::Error>> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("Reddiepoint")
            .repo_name("RedAlt-Steam-Update-Creator")
            .bin_name("RedAlt-Steam-Update-Creator")
            .target("")
            .show_download_progress(true)
            .show_output(true)
            // .no_confirm(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()?
            .update()?;
        println!("Update status: `{}`!", status.version());

        Ok(())
    }
}