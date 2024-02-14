use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use crate::modules::compression::CompressionSettings;

#[derive(Clone, Deserialize, Serialize)]
pub struct SevenZipSettings {
    pub path: Option<PathBuf>,
    pub password: String,
    // Compression settings
    pub archive_format: String,
    pub compression_level: u8,
    pub compression_method: String,
    pub dictionary_size: u16,
    pub word_size: u32,
    pub solid_block_size: u32,
    pub solid_block_size_unit: String,
    pub number_of_cpu_threads: u8,
    pub split_size: u16,
    pub split_size_unit: String
}

impl Default for SevenZipSettings {
    fn default() -> Self {
        Self {
            path: {
                let paths = CompressionSettings::get_detected_paths();
                match paths.first() {
                    Some(Some(path)) => Some(path.into()),
                    _ => None,
                }
            },
            password: String::new(),
            archive_format: "7z".to_string(),
            compression_level: 9,
            compression_method: "LZMA2".to_string(),
            dictionary_size: 64,
            word_size: 273,
            solid_block_size: 16,
            solid_block_size_unit: "g".to_string(),
            number_of_cpu_threads: thread::available_parallelism().unwrap().get() as u8,
            split_size: 0,
            split_size_unit: "g".to_string(),
        }
    }
}

impl SevenZipSettings {
    pub fn compress(&self, download_path: String,
                    input_window_opened_sender: Sender<bool>,
                    stdin_receiver: Receiver<String>,
                    stdout_sender: Sender<String>) -> std::io::Result<()> {
        let _ = stdout_sender.send("\nCompressing files with 7z...\n".to_string());
        let archiver_path = self.path.as_ref().unwrap().to_str().unwrap();
        let mut command = Command::new(archiver_path);
        let _ = std::fs::remove_dir_all(format!("{}/.DepotDownloader", download_path));
        let _ = std::fs::create_dir("./completed");
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("a")
            .arg(format!("-w{}\\completed", std::env::current_dir().unwrap().to_str().unwrap()))
            .arg(format!("-mx{}", self.compression_level))
            .arg(format!("-md{}m", self.dictionary_size))
            .arg(format!("-mfb{}", self.word_size))
            .arg(format!("-ms{}{}", self.solid_block_size, self.solid_block_size_unit))
            .arg(format!("-mmt{}", self.number_of_cpu_threads));
        if self.split_size > 0 {
            command.arg(format!("-v{}{}", self.split_size, self.split_size_unit));
        }
        if !self.password.is_empty() {
            command.arg(format!("-p{}", self.password));
        }
        command
            .arg(format!(".\\completed\\{}.7z", download_path.split('/').last().unwrap()))
            .arg(download_path);
        println!("Command: {:?}", command);
        let mut child = command.spawn()?;

        let result = Arc::new(Mutex::new(Err(std::io::Error::new(std::io::ErrorKind::Other, "Unknown error"))));

        thread::scope(|s| {
            if let Some(mut stderr) = child.stderr.take() {
                let stdo_sender = stdout_sender.clone();
                let input_window_opened_sender = input_window_opened_sender.clone();
                s.spawn(move || {
                    let mut buffer = [0; 1024];
                    loop {
                        match stderr.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                            }
                            _ => break,
                        }
                    }
                });
            }

            if let Some(mut stdout) = child.stdout.take() {
                let stdo_sender = stdout_sender.clone();
                let input_window_opened_sender = input_window_opened_sender.clone();
                s.spawn(move || {
                    let mut buffer = [0; 1024];
                    loop {
                        match stdout.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                            }
                            _ => break,
                        }
                    }
                });
            }

            let stdin = Arc::new(Mutex::new(child.stdin.take().expect("Failed to take stdin")));
            let result_clone = Arc::clone(&result);
            s.spawn(move || {
                loop {
                    match child.try_wait() {
                        Ok(Some(_exit_status)) => {
                            *result_clone.lock().unwrap() = Ok(());
                            break;
                        },
                        Ok(None) => {
                            match stdin_receiver.try_recv() {
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
                }
            });
        });

        Arc::into_inner(result).unwrap().into_inner().unwrap()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WinRARSettings {
    pub path: Option<PathBuf>,
    pub password: String,
    // Compression settings
    pub archive_format: String,
    pub compression_level: u8,
    pub dictionary_size: u16,
    pub solid: bool,
    pub number_of_cpu_threads: u8,
    pub split_size: u16,
    pub split_size_unit: String
}

impl Default for WinRARSettings {
    fn default() -> Self {
        Self {
            path: {
                let paths = CompressionSettings::get_detected_paths();
                match paths.get(1) {
                    Some(Some(path)) => Some(path.into()),
                    _ => None,
                }
            },
            password: String::new(),
            archive_format: "rar".to_string(),
            compression_level: 5,
            dictionary_size: 512,
            solid: true,
            number_of_cpu_threads: thread::available_parallelism().unwrap().get() as u8,
            split_size: 0,
            split_size_unit: "g".to_string(),
        }
    }
}

impl WinRARSettings {
    pub fn compress(&self, download_path: String, input_window_opened_sender: Sender<bool>,
                    stdin_receiver: Receiver<String>,
                    stdo_sender: Sender<String>) -> std::io::Result<()> {
        let _ = stdo_sender.send("\nCompressing files with WinRAR...\n".to_string());
        let archiver_path = self.path.as_ref().unwrap().to_str().unwrap();
        let mut command = Command::new(archiver_path);
        let _ = std::fs::remove_dir_all(format!("{}/.DepotDownloader", download_path));
        let _ = std::fs::create_dir("./completed");
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("a")
            .arg(format!("-w{}\\completed", std::env::current_dir().unwrap().to_str().unwrap()))
            .arg(format!("-m{}", self.compression_level))
            .arg(format!("-md{}m", self.dictionary_size))
            .arg(format!("-mt{}", self.number_of_cpu_threads));
        if archiver_path.contains("WinRAR.exe") {
            command.arg(format!("-af{}", self.archive_format));
        }
        if self.solid {
            command.arg(format!("-s{}", if self.split_size > 0 { "v-" } else { "" }));
        }
        if self.split_size > 0 {
            command.arg(format!("-v{}{}", self.split_size, self.split_size_unit));
        }
        if !self.password.is_empty() {
            command.arg(format!("-p{}", self.password));
        }
        command
            .arg("-ep1")
            .arg(format!(".\\completed\\{}.rar", download_path.split('/').last().unwrap()))
            .arg(download_path);
        println!("Command: {:?}", command);
        let mut child = command.spawn()?;

        let result = Arc::new(Mutex::new(Err(std::io::Error::new(std::io::ErrorKind::Other, "Unknown error"))));

        thread::scope(|s| {
            if let Some(mut stderr) = child.stderr.take() {
                let stdo_sender = stdo_sender.clone();
                let input_window_opened_sender = input_window_opened_sender.clone();
                s.spawn(move || {
                    let mut buffer = [0; 1024];
                    loop {
                        match stderr.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                            }
                            _ => break,
                        }
                    }
                });
            }

            if let Some(mut stdout) = child.stdout.take() {
                let stdo_sender = stdo_sender.clone();
                let input_window_opened_sender = input_window_opened_sender.clone();
                s.spawn(move || {
                    let mut buffer = [0; 1024];
                    loop {
                        match stdout.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let _ = stdo_sender.send(String::from_utf8_lossy(&buffer[..n]).parse().unwrap());
                            }
                            _ => break,
                        }
                    }
                });
            }

            let stdin = Arc::new(Mutex::new(child.stdin.take().expect("Failed to take stdin")));
            let result_clone = Arc::clone(&result);
            s.spawn(move || {
                loop {
                    match child.try_wait() {
                        Ok(Some(_exit_status)) => {
                            *result_clone.lock().unwrap() = Ok(());
                            break;
                        },
                        Ok(None) => {
                            match stdin_receiver.try_recv() {
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
                }
            });
        });

        Arc::into_inner(result).unwrap().into_inner().unwrap()
    }
}