use std::process::Command;
use crate::modules::changes::Changes;

pub struct DepotDownloaderSettings {
    pub username: String,
    pub password: String,
    pub remember_credentials: bool,
}

impl Default for DepotDownloaderSettings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            remember_credentials: true,
        }
    }
}

fn write_changes_to_file(changes: &Changes) -> std::io::Result<()> {
    let download_files = changes.added.join("\n") + &changes.modified.join("\n");
    // Write changes to file files.txt
    let path = "files.txt";
    std::fs::write(path, download_files)?;
    Ok(())
}
pub fn download_changes(changes: &Changes, settings: &DepotDownloaderSettings) -> std::io::Result<()> {
    write_changes_to_file(changes)?;
    // Run Depot Downloader
    // Command::new("./DepotDownloader.exe")
    //     .args()

    Ok(())
}
