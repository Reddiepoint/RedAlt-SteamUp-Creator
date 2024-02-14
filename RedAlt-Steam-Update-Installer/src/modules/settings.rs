use std::path::PathBuf;

pub struct Settings {
    create_backup: bool,
    changes_directory: Option<PathBuf>
}