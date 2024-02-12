use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use crate::modules::changes::Changes;


#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum OSType {
    Windows,
    Linux,
    Mac,
    Current
}

impl Display for OSType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OSType::Windows => "Windows",
            OSType::Linux => "Linux",
            OSType::Mac => "Mac",
            OSType::Current => "Current OS",
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DepotDownloaderSettings {
    // Used by Depot Downloader
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub os: OSType,
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
            os: OSType::Current,
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

pub fn download_changes(changes: &Changes, settings: &DepotDownloaderSettings,
                        input_window_opened_sender: Sender<bool>,
                        input_receiver: Receiver<String>,
                        stdo_sender: Sender<String>,
                        status_sender: Sender<std::io::Result<String>>)
                        -> std::io::Result<()> {
    write_changes_to_file(changes)?;
    let _ = stdo_sender.clone().send("Starting Depot Downloader...\n".to_string());
    // Download path
    let path = format!("./downloads/{} ({}) [Build {} to {}]", changes.app, changes.depot, changes.initial_build, changes.final_build);
    // Run Depot Downloader
    let mut command = Command::new("./DepotDownloader.exe");
    command
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-app", &changes.app, "-depot", &changes.depot, "-manifest", &changes.manifest])
        .args(["-dir",
            &path])
        .args(["-filelist", "files.txt"]);

    match settings.remember_credentials {
        true => if !settings.password.is_empty() {
            command.args(["-username", &settings.username, "-password", &settings.password, "-remember-password"])
        } else {
            command.args(["-username", &settings.username, "-remember-password"])
        },
        false => command.args(["-username", &settings.username, "-password", &settings.password])
    };

    match settings.os {
        OSType::Windows => { command.args(["-os", "windows"]); },
        OSType::Linux => { command.args(["-os", "linux"]); },
        OSType::Mac => { command.args(["-os", "macos"]); },
        OSType::Current => {}
    }

    command
        .args(["-max-servers", &settings.max_servers.to_string()])
        .args(["-max-downloads", &settings.max_downloads.to_string()]);

    let mut child = command.spawn()?;

    let patterns = [
        "STEAM GUARD! Please enter the auth code",
        "Enter account password"
    ];

    if let Some(mut stdout) = child.stdout.take() {
        let stdo_sender = stdo_sender.clone();
        let input_window_opened_sender = input_window_opened_sender.clone();
        thread::spawn(move || {
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

    if let Some(mut stderr) = child.stderr.take() {
        let stdo_sender = stdo_sender.clone();
        let input_window_opened_sender = input_window_opened_sender.clone();
        thread::spawn(move || {
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

    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("Failed to take stdin")));

    thread::spawn(move || {
        loop {
            match child.try_wait() {
                Ok(Some(_exit_status)) => {
                    let _ = status_sender.send(Ok(path));
                    let _ = stdo_sender.send("Depot Downloader exited.\n".to_string());
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
                }
            }
        }
    });

    Ok(())
}
