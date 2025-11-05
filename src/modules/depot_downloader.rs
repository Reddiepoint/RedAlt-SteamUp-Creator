use crate::modules::changes::{Changelog, Changes};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::create_dir;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Deserialize, Serialize)]
pub struct EncryptionKey {
    pub encrypted_encryption_key: [u8; 32],
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DepotDownloaderSettings {
    #[serde(skip)]
    pub encryption_key: EncryptionKey,
    pub username_nonce: [u8; 12],
    pub encrypted_username: Vec<u8>,
    // Used by Depot Downloader
    #[serde(skip)]
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub max_downloads: u8,
    // Used by app
    pub remember_credentials: bool,
    #[serde(skip)]
    pub download_manifest: bool,
    #[serde(skip)]
    pub download_entire_depot: bool,
    #[serde(skip)]
    pub depot_downloader_input_window_opened: bool,
    #[serde(skip)]
    pub input: String,
}

impl Default for DepotDownloaderSettings {
    fn default() -> Self {
        Self {
            encryption_key: EncryptionKey {
                encrypted_encryption_key: [0; 32],
            },
            username_nonce: [0; 12],
            encrypted_username: Vec::new(),
            username: String::new(),
            password: String::new(),
            max_downloads: 8,
            remember_credentials: true,
            download_manifest: true,
            download_entire_depot: false,
            depot_downloader_input_window_opened: false,
            input: String::new(),
        }
    }
}

fn write_changes_to_file(changes: &Changelog) -> std::io::Result<()> {
    let download_files = changes.added.join("\n") + "\n" + &changes.modified.join("\n");
    // Write changes to file files.txt
    let path = "files.txt";
    std::fs::write(path, download_files)?;
    Ok(())
}

pub fn download_changes(
    changes: &Changes,
    changelog: &Changelog,
    settings: &DepotDownloaderSettings,
    target_os: String,
    input_window_opened_sender: Sender<bool>,
    input_receiver: Receiver<String>,
    output_sender: Sender<String>,
) -> std::io::Result<PathBuf> {
    write_changes_to_file(changelog)?;
    let _ = output_sender
        .clone()
        .send("Starting Depot Downloader...\n".to_string());
    // Download directory
    let base_path = current_dir().unwrap().to_path_buf().join("Downloads").join(
        match settings.download_entire_depot {
            true => {
                format!(
                    "{} (Build {}) {}",
                    &changes.app_name, changes.initial_build, target_os
                )
            }
            false => {
                format!(
                    "{} (Build {} to {}) {}",
                    &changes.app_name, &changes.initial_build, &changes.final_build, target_os
                )
            }
        },
    );

    let download_path = base_path;
    let download_path_clone = download_path.clone();
    // Run Depot Downloader
    let mut command = Command::new("./DepotDownloader");
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args([
            "-app",
            &changes.app_id,
            "-depot",
            &changelog.depot_id,
            "-manifest",
            &changelog.manifest,
        ])
        .args(["-dir", download_path.to_str().unwrap()]);

    if !settings.download_entire_depot {
        command.args(["-filelist", "files.txt"]);
    }

    match settings.remember_credentials {
        true => {
            if !settings.password.is_empty() {
                command.args([
                    "-username",
                    &settings.username,
                    "-password",
                    &settings.password,
                    "-remember-password",
                ])
            } else {
                command.args(["-username", &settings.username, "-remember-password"])
            }
        }
        false => command.args([
            "-username",
            &settings.username,
            "-password",
            &settings.password,
        ]),
    };

    command.args(["-max-downloads", &settings.max_downloads.to_string()]);

    let mut child = command.spawn()?;
    // let _ = output_sender.send("Depot Downloader started.\n".to_string());

    let patterns = [
        "STEAM GUARD! Please enter the auth code",
        "Enter account password",
    ];

    let result = Arc::new(Mutex::new(Err(std::io::Error::other("Unknown error"))));

    thread::scope(|s| {
        // Write error to output window and check if Steam Guard code is required
        if let Some(mut stderr) = child.stderr.take() {
            let stdo_sender = output_sender.clone();
            let input_window_opened_sender = input_window_opened_sender.clone();
            s.spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let _ = stdo_sender
                                .send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());

                            for pattern in patterns {
                                if String::from_utf8_lossy(&buffer[..n]).contains(pattern) {
                                    input_window_opened_sender.send(true).unwrap();
                                }
                            }
                        }
                        _ => break,
                    }
                }
            });
        }

        if let Some(mut stdout) = child.stdout.take() {
            let stdo_sender = output_sender.clone();
            let input_window_opened_sender = input_window_opened_sender.clone();
            s.spawn(move || {
                let mut buffer = [0; 1024];
                let mut overflow = String::new();
                loop {
                    match stdout.read(&mut buffer) {
                        Ok(0) => {
                            if !overflow.is_empty() {
                                let _ = stdo_sender.send(overflow.clone());
                                if patterns.iter().any(|pattern| overflow.contains(pattern)) {
                                    let _ = input_window_opened_sender.send(true);
                                }
                            }
                            break;
                        }
                        Ok(n) => {
                            overflow.push_str(String::from_utf8_lossy(&buffer[..n]).as_ref());
                            // let _ = stdo_sender
                            //     .send(String::from_utf8_lossy(&buffer[..n]).to_string());
                            let mut lines = overflow.lines();
                            let mut last_line = String::new();
                            while let Some(line) = lines.next() {
                                last_line = line.to_string();
                                if let Some(_) = lines.clone().next() {
                                    let _ = stdo_sender.send(line.trim().to_string());
                                    if patterns.iter().any(|p| line.contains(p)) {
                                        let _ = input_window_opened_sender.send(true);
                                    }
                                }
                            }
                            if overflow.ends_with('\n') {
                                let _ = stdo_sender.send(last_line.clone());
                                if patterns.iter().any(|p| last_line.contains(p)) {
                                    let _ = input_window_opened_sender.send(true);
                                }
                                overflow.clear();
                            } else {
                                overflow = last_line;
                            }
                        }
                        _ => break,
                    }
                }
            });
        }

        let stdin = Arc::new(Mutex::new(
            child.stdin.take().expect("Failed to take stdin"),
        ));
        let result_clone = Arc::clone(&result);
        s.spawn(move || loop {
            match child.try_wait() {
                Ok(Some(_exit_status)) => {
                    *result_clone.lock().unwrap() = Ok(download_path_clone);
                    break;
                }
                Ok(None) => match input_receiver.try_recv() {
                    Ok(code) => {
                        let stdin = stdin.clone();
                        let code = format!("{code}\n");
                        stdin
                            .lock()
                            .expect("Failed to lock stdin")
                            .write_all(code.as_bytes())
                            .expect("Failed to write to stdin");
                        stdin
                            .lock()
                            .expect("Failed to lock stdin")
                            .flush()
                            .expect("Failed to flush stdin");
                    }
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                },
                Err(error) => {
                    *result_clone.lock().unwrap() = Err(error);
                    break;
                }
            }
        });
    });
    if settings.download_manifest {
        let _ = output_sender.send("Downloading manifest...\n".to_string());
        let installer_path = download_path.join("*RedAlt-SteamUp-Installer");
        let _ = create_dir(&installer_path);
        let _ = download_manifest(&download_path, changes, changelog, settings);
        let _ = std::fs::rename(
            download_path.join(format!(
                "manifest_{}_{}.txt",
                changelog.depot_id, changelog.manifest,
            )),
            installer_path.join(format!(
                "manifest_{}_{}.txt",
                changelog.depot_id, changelog.manifest,
            )),
        );
        let depot_downloader_path = download_path.join(".DepotDownloader");
        let _ = std::fs::rename(
            depot_downloader_path.join(format!(
                "{}_{}.manifest",
                changelog.depot_id, changelog.manifest,
            )),
            installer_path.join(format!(
                "{}_{}.manifest",
                changelog.depot_id, changelog.manifest,
            )),
        );
        let _ = output_sender.send("Downloaded manifest.\n".to_string());
    }
    Arc::into_inner(result).unwrap().into_inner().unwrap()
}

pub fn download_manifest(
    download_path: &PathBuf,
    changes: &Changes,
    changelog: &Changelog,
    settings: &DepotDownloaderSettings,
) -> std::io::Result<()> {
    // Run Depot Downloader
    // src/modules/create_update.rs:349
    let mut command = Command::new("./DepotDownloader");
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args([
            "-app",
            &changes.app_id,
            "-depot",
            &changelog.depot_id,
            "-manifest",
            &changelog.manifest,
        ])
        .args(["-dir", download_path.to_str().unwrap()])
        .arg("-manifest-only");

    match settings.remember_credentials {
        true => {
            if !settings.password.is_empty() {
                command.args([
                    "-username",
                    &settings.username,
                    "-password",
                    &settings.password,
                    "-remember-password",
                ])
            } else {
                command.args(["-username", &settings.username, "-remember-password"])
            }
        }
        false => command.args([
            "-username",
            &settings.username,
            "-password",
            &settings.password,
        ]),
    };

    let mut child = command.spawn()?;
    let _ = child.wait();
    Ok(())
}
