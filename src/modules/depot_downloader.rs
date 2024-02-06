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

pub fn download_changes(changes: &Changes, settings: &DepotDownloaderSettings) {
    todo!()
}
