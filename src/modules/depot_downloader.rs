use std::fmt::format;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{Ui, Window};
use crate::modules::changes::Changes;

#[derive(Clone)]
pub struct DepotDownloaderSettings {
    pub username: String,
    pub password: String,
    pub remember_credentials: bool,
    pub steam_guard_code_window_opened: bool,
    pub steam_guard_code: String,
}

impl Default for DepotDownloaderSettings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            remember_credentials: true,
            steam_guard_code_window_opened: false,
            steam_guard_code: String::new()
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
                        steam_guard_code_window_opened_sender: Sender<bool>, steam_guard_code_receiver: Receiver<String>)
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
    println!("{:?}", command);
    match settings.remember_credentials {
        true => if !settings.username.is_empty() && !settings.password.is_empty() {
            command.args(["-username", &settings.username, "-password", &settings.password, "-remember-password"])
        } else {
            command.args(["-username", "-remember-password"])
        },
        false => command.arg("-username").arg(&settings.username).arg("-password").arg(&settings.password)
    };
    command.args(["-dir", "./downloads", "-filelist", "files.txt"]);

    println!("Spawned");
    let mut child = command.spawn().unwrap();

    if let Some(mut stdout) = child.stdout.take() {
        thread::spawn(move || {
            let mut buffer = [0; 256];
            loop {
                match stdout.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        println!("Output: {}", String::from_utf8_lossy(&buffer[..n]));
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
                        println!("Error: {}", String::from_utf8_lossy(&buffer[..n]));
                        if String::from_utf8_lossy(&buffer[..n]).contains("STEAM GUARD!") {
                            steam_guard_code_window_opened_sender.send(true).unwrap();
                        }
                    }
                    _ => break,
                }
            }
        });
    }

    if let Some(mut stdin) = child.stdin.take() {
        println!("Stdin taken");
        thread::spawn(move || {
            loop {
                match steam_guard_code_receiver.try_recv() {
                    Ok(code) => {
                        println!("Auth code received: {}", code);
                        let mut input = code;
                        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
                        break; // Exit the loop after writing to stdin
                    }
                    Err(_) => {
                        // No code received, continue looping
                        std::thread::sleep(std::time::Duration::from_millis(100)); // Add a small delay to avoid busy-waiting
                    }
                }
            }
        });
    }


    child.wait();
    Ok(())
}
