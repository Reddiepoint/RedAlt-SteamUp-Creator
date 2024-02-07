use std::fmt::format;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui::{Ui, Window};
use crate::modules::changes::Changes;

#[derive(Clone)]
pub struct DepotDownloaderSettings {
    pub username: String,
    pub password: String,
    pub remember_credentials: bool,
    pub depot_downloader_input_window_opened: bool,
    pub input: String,
}

impl Default for DepotDownloaderSettings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            remember_credentials: true,
            depot_downloader_input_window_opened: false,
            input: String::new()
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
                        stdo_sender: Sender<String>)
                        -> std::io::Result<()> {
    write_changes_to_file(changes)?;
    // Run Depot Downloader
    // Create command
    let mut command = Command::new("./DepotDownloader.exe");
    command
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-app", &changes.app, "-depot", &changes.depot, "-manifest", &changes.manifest]);

    match settings.remember_credentials {
        true => if !settings.password.is_empty() {
            command.args(["-username", &settings.username, "-password", &settings.password, "-remember-password"])
        } else {
            command.args(["-username", &settings.username, "-remember-password"])
        },
        false => command.args(["-username", &settings.username, "-password", &settings.password])
    };
    command.args(["-dir",
        &format!("./downloads/{} ({}) [{} to {}]", changes.app, changes.depot, changes.initial_build, changes.final_build),
        "-filelist", "files.txt"]);

    let mut child = command.spawn().unwrap();

    let patterns = [
        "STEAM GUARD! Please enter the auth code",
        "Enter account password"
    ];

    if let Some(mut stdout) = child.stdout.take() {
        let depot_downloader_stdio_sender = stdo_sender.clone();
        let depot_downloader_window_opened_sender = input_window_opened_sender.clone();
        thread::spawn(move || {
            let mut buffer = [0; 256];
            loop {
                match stdout.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        depot_downloader_stdio_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());

                        for pattern in patterns {
                            if String::from_utf8_lossy(&buffer[..n]).contains(pattern) {
                                depot_downloader_window_opened_sender.send(true).unwrap();
                            }
                        }
                    }
                    _ => break,
                }
            }
        });
    }

    if let Some(mut stderr) = child.stderr.take() {
        thread::spawn(move || {
            let mut buffer = [0; 256];
            loop {
                match stderr.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        // println!("Error: {}", String::from_utf8_lossy(&buffer[..n]));
                        stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());

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
                Ok(Some(_status)) => {
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
