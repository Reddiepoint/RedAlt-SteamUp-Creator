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
                        steam_guard_code_window_opened_sender: Sender<bool>,
                        steam_guard_code_receiver: Receiver<String>,
                        depot_downloader_stdio_sender: Sender<String>)
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

    println!("Spawned");
    println!("{:?}", command);
    let mut child = command.spawn().unwrap();

    if let Some(mut stdout) = child.stdout.take() {
        let depot_downloader_stdio_sender = depot_downloader_stdio_sender.clone();
        thread::spawn(move || {
            let mut buffer = [0; 256];
            loop {
                match stdout.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        depot_downloader_stdio_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                    }
                    _ => break,
                }
            }
        });
    }

    if let Some(mut stderr) = child.stderr.take() {
        let depot_downloader_stdio_sender = depot_downloader_stdio_sender.clone();
        thread::spawn(move || {
            let mut buffer = [0; 256];
            loop {
                match stderr.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        // println!("Error: {}", String::from_utf8_lossy(&buffer[..n]));
                        depot_downloader_stdio_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                        if String::from_utf8_lossy(&buffer[..n]).contains("STEAM GUARD!") {
                            steam_guard_code_window_opened_sender.send(true).unwrap();
                        }
                    }
                    _ => break,
                }
            }
        });
    }

    /*if let Some(mut stdin) = child.stdin.take() {
        println!("Stdin taken");
        thread::spawn(move || {
            loop {
                match steam_guard_code_receiver.try_recv() {
                    Ok(code) => {
                        println!("Auth code received: {}", code);
                        stdin.write_all(code.as_bytes()).expect("Failed to write to stdin");
                        println!("Wrote to stdin");
                        break; // Closes stdin pipe
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(TryRecvError::Disconnected) => {
                        println!("Channel has been disconnected");
                        break;
                    }
                }
            }
        });
    }*/
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("Failed to take stdin")));

    // let child= Arc::new(Mutex::new(child));
    // let child_clone = child.clone();
    thread::spawn(move || {
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    println!("Exited");
                    break;
                },
                Ok(None) => {
                    println!("Still running");
                    match steam_guard_code_receiver.try_recv() {
                        Ok(code) => {
                            let stdin = stdin.clone();
                            let code = format!("{}\n", code);
                            println!("Auth code received: {}", code);
                            stdin.lock().expect("Failed to lock stdin").write_all(code.as_bytes()).expect("Failed to write to stdin");
                            println!("Wrote to stdin");
                            stdin.lock().expect("Failed to lock stdin").flush().expect("Failed to flush stdin");
                        }
                        Err(TryRecvError::Empty) => {
                            println!("Still running");
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(TryRecvError::Disconnected) => {
                            println!("Channel has been disconnected");
                        }
                    }
                }
                Err(error) => {
                    eprintln!("error: {}", error);
                }
            }

        }
    });


    // child.lock().expect("Failed to lock child").wait().expect("Failed to wait for child");

    Ok(())
}
