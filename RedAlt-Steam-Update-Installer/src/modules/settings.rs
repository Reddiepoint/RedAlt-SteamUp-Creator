use std::env::current_dir;
use std::fmt::Display;
use std::fs::{create_dir, read_dir};
use std::io::ErrorKind;
use std::path::PathBuf;
use crate::get_input;
use crate::modules::changes::Changes;

pub struct Settings {
    pub changes_file: Option<PathBuf>,
    game_directory: Option<PathBuf>,
    files_directory: Option<PathBuf>,
    backup_directory: Option<PathBuf>,
    create_backup: bool,
    copy_files: bool,
    remove_files: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            changes_file: {
                let mut files = read_dir(current_dir().unwrap()).unwrap();
                files
                    .find(|file| file.as_ref().unwrap().file_name().to_str().unwrap().contains("changes.json"))
                    .map(|file| file.unwrap().path())
            },
            game_directory: {
                let current_directory = current_dir().unwrap();
                let parent = current_directory.parent().unwrap();
                if let Some(grandparent) = parent.parent() {
                    Some(grandparent.to_path_buf())
                } else {
                    Some(parent.to_path_buf())
                }
            },
            files_directory: Some(current_dir().unwrap().parent().unwrap().to_path_buf()),
            backup_directory: None,
            create_backup: true,
            copy_files: true,
            remove_files: true,
        }
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Current settings:")?;
        writeln!(f, "Using changes file (changes_file): {}", match &self.changes_file {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "Using game directory (game_directory): {}", match &self.game_directory {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "Using update directory (files_directory): {}", match &self.files_directory {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "Create backup (create_backup): {}", self.create_backup)?;
        writeln!(f, "Copy files (copy_files): {}", self.copy_files)?;
        write!(f, "Remove files (remove_files): {}", self.remove_files)?;
        Ok(())
    }
}

impl Settings {
    pub fn modify_fields(&mut self, input: String) {
        let input = input.split(' ').collect::<Vec<&str>>();
        let field = match input.get(1) {
            Some(field) => field.to_owned(),
            None => {
                println!("Enter a field.");
                return;
            }
        };
        // Rest of input
        // let value = input[2..].join(" ").replace('"', "").trim().to_string();
        let value = match input.get(2) {
            Some(_) => { input[2..].join(" ").replace('"', "").trim().to_string() }
            None => {
                println!("Enter a value.");
                return;
            }
        };
        let parse_bool = |value: &str| {
            let value = value.to_lowercase();
            if ["true", "t"].contains(&value.as_str()) {
                Some(true)
            } else if ["false", "f"].contains(&value.as_str()) {
                Some(false)
            } else {
                None
            }
        };
        match field {
            "changes_file" => self.changes_file = Some(PathBuf::from(value)),
            "game_directory" => self.game_directory = Some(PathBuf::from(value)),
            "files_directory" => self.files_directory = Some(PathBuf::from(value)),
            "create_backup" => match parse_bool(&value) {
                Some(value) => { self.create_backup = value },
                None => { println!("Invalid value") }
            },
            "copy_files" => match parse_bool(&value) {
                Some(value) => { self.copy_files = value },
                None => { println!("Invalid value") }
            },
            "remove_files" => match parse_bool(&value) {
                Some(value) => { self.remove_files = value },
                None => { println!("Invalid value") }
            },
            _ => println!("Field not found"),
        }

        println!("{}", self);
    }

    pub fn update_game(&mut self) {
        if self.game_directory.is_none() {
            println!("Provide a game directory.");
            return;
        };

        let mut changes = match Changes::parse_changes(&self.changes_file) {
            Some(changes) => changes,
            None => return
        };

        println!("Updating {} with files in {} from {}.\n",
                 self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap(),
                 self.files_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap(),
                 self.changes_file.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
        println!("{}", self);

        let input = get_input("Continue? [y/N] ");
        match input.to_lowercase().as_str() {
            "y" | "yes" => {
                if self.create_backup {
                    let backup_directory = self.game_directory.as_ref().unwrap().to_str().unwrap().to_string()
                        + "\\.Backup";
                    self.backup_directory = Some(backup_directory.into());
                    if let Err(error) = create_dir(self.backup_directory.as_ref().unwrap()) {
                        if error.kind() != ErrorKind::AlreadyExists {
                            println!("Error creating backups directory: {}", error);
                            return;
                        }
                    };
                }
                if self.copy_files {
                    self.copy_files(&mut changes);
                }
                if self.remove_files {
                    self.remove_files(&mut changes);
                }
            },
            _ => {
                println!("Cancelled update.");
            }
        };

        println!("Finished updating.");
    }

    fn copy_files(&self, changes: &mut Changes) {
        let mut new_files = vec![];
        new_files.append(&mut changes.added);
        new_files.append(&mut changes.modified);
        for path in &new_files {
            if path.contains(".RedAlt-Steam-Installer") {
                continue;
            }

            println!("Copying {} to {}", path, self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
            let new_file = self.files_directory.as_ref().unwrap().to_str().unwrap().to_string() + "\\" + path;
            let old_file = self.game_directory.as_ref().unwrap().to_str().unwrap().to_string() + "\\" + path;
            if self.create_backup {
                let backup_file = self.backup_directory.as_ref().unwrap().to_str().unwrap().to_string() + "\\" + path;
                if let Err(error) = std::fs::copy(&old_file, backup_file) {
                    if error.kind() == ErrorKind::PermissionDenied {
                        println!("Error copying to backup folder: {}", error);
                        return;
                    }
                }
            }
            if let Err(error) = std::fs::copy(&new_file, &old_file) {
                if error.kind() == ErrorKind::PermissionDenied {
                    println!("Error copying to game folder: {}", error);
                    return;
                }
            }
        }
    }

    fn remove_files(&self, changes: &mut Changes) {
        for path in &changes.removed {
            println!("Removing {} from {}", path, self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
            let old_file = self.game_directory.as_ref().unwrap().to_str().unwrap().to_string() + "\\" + path;
            if self.create_backup {
                let backup_file = self.backup_directory.as_ref().unwrap().to_str().unwrap().to_string() + "\\" + path;
                if let Err(error) = std::fs::copy(&old_file, backup_file) {
                    if error.kind() == ErrorKind::PermissionDenied {
                        println!("Error copying to backup folder: {}", error);
                        return;
                    }
                }
            }
            if let Err(error) = std::fs::remove_file(&old_file) {
                if error.kind() == ErrorKind::PermissionDenied {
                    println!("Error removing file: {}", error);
                    return;
                }
            }
        }
    }

    pub fn show_changes(&self) {
        let mut changes = match Changes::parse_changes(&self.changes_file) {
            Some(changes) => changes,
            None => return
        };

        println!("Changes for {} (Depot {} - Manifest {}):", changes.app, changes.depot, changes.manifest);
        println!("Initial Build: {}+", changes.initial_build);
        println!("Final Build: {}", changes.final_build);
        let display_vec = |vec: &Vec<String>| {
            vec.iter().map(|value| format!("  {}", value)).collect::<Vec<String>>().join("\n")
        };
        println!("Added:\n{}", display_vec(&changes.added));
        println!("Removed:\n{}", display_vec(&changes.removed));
        println!("Modified:\n{}", display_vec(&changes.modified));
    }
}
