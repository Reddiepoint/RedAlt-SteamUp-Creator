use crate::modules::changes::Changes;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DepotDownloaderSettings {
    // Used by Depot Downloader
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub max_servers: u8,
    pub max_downloads: u8,
    // Used by app
    pub remember_credentials: bool,
    #[serde(skip)]
    pub depot_downloader_input_window_opened: bool,
    #[serde(skip)]
    pub input: String
}

impl Default for DepotDownloaderSettings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            max_servers: 20,
            max_downloads: 8,
            remember_credentials: true,
            depot_downloader_input_window_opened: false,
            input: String::new(),
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

pub fn download_changes(
    changes: &Changes,
    settings: &DepotDownloaderSettings,
    input_window_opened_sender: Sender<bool>,
    input_receiver: Receiver<String>,
    output_sender: Sender<String>,
    download_entire_depot: bool,
) -> std::io::Result<String> {
    write_changes_to_file(changes)?;
    let _ = output_sender.clone().send("Starting Depot Downloader...\n".to_string());
    // Download path
    let path = format!(
        "./Downloads/{} ({}) [Build {} to {}]",
        changes.app, changes.depot, changes.initial_build, changes.final_build
    );
    // Run Depot Downloader
    let mut command = Command::new("./DepotDownloader.exe");
    command
        .creation_flags(0x08000000)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["-app", &changes.app, "-depot", &changes.depot, "-manifest", &changes.manifest])
        .args(["-dir", &path]);

    if !download_entire_depot {
        command.args(["-filelist", "files.txt"]);
    }

    match settings.remember_credentials {
        true => if !settings.password.is_empty() {
            command.args(["-username", &settings.username, "-password", &settings.password, "-remember-password"])
        } else {
            command.args(["-username", &settings.username, "-remember-password"])
        },
        false => command.args(["-username", &settings.username, "-password", &settings.password])
    };

    command
        .args(["-max-servers", &settings.max_servers.to_string()])
        .args(["-max-downloads", &settings.max_downloads.to_string()]);

    let mut child = command.spawn()?;
    // let _ = output_sender.send("Depot Downloader started.\n".to_string());

    let patterns = [
        "STEAM GUARD! Please enter the auth code",
        "Enter account password",
    ];

    let result = Arc::new(Mutex::new(Err(std::io::Error::new(std::io::ErrorKind::Other, "Unknown error"))));

    thread::scope(|s| {
        if let Some(mut stderr) = child.stderr.take() {
            let stdo_sender = output_sender.clone();
            let input_window_opened_sender = input_window_opened_sender.clone();
            s.spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());

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
                loop {
                    match stdout.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());

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

        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("Failed to take stdin")));
        let result_clone = Arc::clone(&result);
        s.spawn(move || loop {
            match child.try_wait() {
                Ok(Some(_exit_status)) => {
                    *result_clone.lock().unwrap() = Ok(path.clone());
                    break;
                },
                Ok(None) => {
                    match input_receiver.try_recv() {
                        Ok(code) => {
                            let stdin = stdin.clone();
                            let code = format!("{}\n", code);
                            stdin.lock().expect("Failed to lock stdin").write_all(code.as_bytes()).expect("Failed to write to stdin");
                            stdin.lock().expect("Failed to lock stdin").flush().expect("Failed to flush stdin");
                        },
                        Err(_) => {
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
                Err(error) => {
                    eprintln!("error: {}", error);
                    *result_clone.lock().unwrap() = Err(error);
                    break;
                }
            }
        });
    });

    Arc::into_inner(result).unwrap().into_inner().unwrap()
}
