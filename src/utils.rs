use colored::Colorize;
use futures_util::StreamExt;
use std::{
    fs::File,
    io::Write,
    io::{self},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const FRAMES: [&str; 12] = [
    "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
];

lazy_static! {
    static ref LAST_FRAME: Mutex<std::time::Instant> = Mutex::new(std::time::Instant::now());
    static ref FRAME_INDEX: Mutex<usize> = Mutex::new(0);
}

fn loading_str(line: &str, index: Option<usize>) -> String {
    let mut frame_index = FRAME_INDEX.lock().unwrap();
    let mut last_frame = LAST_FRAME.lock().unwrap();

    if *frame_index >= FRAMES.len() {
        *frame_index = 0;
    }

    let f_index = if last_frame.elapsed().as_millis() > 100 {
        index.unwrap_or_else(|| {
            *frame_index += 1;
            *frame_index - 1
        })
    } else {
        index.unwrap_or(*frame_index)
    };

    if last_frame.elapsed().as_millis() > 100 {
        *last_frame = std::time::Instant::now();
    }

    line.replace("[ ]", FRAMES[f_index])
}

pub fn terminal_width() -> usize {
    let (width, _) = term_size::dimensions().unwrap_or((80, 24));
    width
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { api_key: None }
    }
}

fn app_dir() -> PathBuf {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    dir.push(".gpt-commit-rust");
    dir
}

fn config_path() -> String {
    let mut dir = app_dir();
    dir.push("config.toml");
    dir.to_str().unwrap().to_owned()
}

impl Config {
    pub fn save(&self) {
        let config = toml::to_string(self).unwrap();
        let dir = app_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir).unwrap();
        }
        let mut file = File::create(config_path()).unwrap();
        file.write_all(config.as_bytes()).unwrap();
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.to_owned().unwrap_or(
            std::env::var("CHAT_GPT_TOKEN")
                .unwrap_or_else(|_| "".to_owned())
                .to_owned(),
        )
    }
}

pub fn get_config() -> Config {
    let config =
        toml::from_str::<Config>(&std::fs::read_to_string(config_path()).unwrap_or_else(|_| {
            let config = &mut Config::default();
            let api_key = std::env::var("CHAT_GPT_TOKEN");
            if api_key.is_ok() {
                let api_key = api_key.unwrap();
                config.set_api_key(api_key);
                config.save();
            }
            toml::to_string(&config).unwrap()
        }))
        .unwrap();
    config
}

pub fn get_executable_name() -> String {
    std::env::current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

pub async fn download_update() -> Result<(), String> {
    let update_url = if cfg!(windows) {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-Windows/gpt-commit-rust.exe")
    } else if cfg!(target_os = "macos") {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-Linux/gpt-commit-rust")
    } else if cfg!(target_os = "linux") {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-macOS/gpt-commit-rust")
    } else {
        return Err("Unsupported OS".to_owned());
    };

    let update_file_path = if cfg!(windows) {
        Path::new(app_dir().as_os_str()).join("gpt-commit-rust-update.exe")
    } else {
        Path::new(app_dir().as_os_str()).join("gpt-commit-rust-update")
    };
    let client = reqwest::Client::new();
    let update = client
        .get(&update_url)
        .send()
        .await
        .or(Err(
            "Failed to download update. Please try again later or download the update manually.",
        ))
        .unwrap();

    let total_size = update.content_length().unwrap();

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap().progress_chars("#>-"));
    progress_bar.set_message("Downloading update");

    let mut downloaded = 0;
    let mut stream = update.bytes_stream();
    let mut update_file = File::create(update_file_path.to_owned()).unwrap();

    while let Some(item) = stream.next().await {
        let item = item.unwrap();
        downloaded += item.len();
        progress_bar.set_position(downloaded as u64);
        update_file.write_all(&item).unwrap();
    }

    progress_bar.finish();

    if cfg!(unix) {
        Command::new("chmod")
            .arg("+x")
            .arg(update_file_path.to_owned().as_mut_os_str())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    version: String,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

pub async fn check_for_update() -> bool {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let update_url = if cfg!(windows) {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-Windows/Cargo.toml")
    } else if cfg!(target_os = "macos") {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-Linux/Cargo.toml")
    } else if cfg!(target_os = "linux") {
        format!("https://github.com/DerTyp7214/gpt-commit-rust/releases/download/latest-macOS/Cargo.toml")
    } else {
        return false;
    };

    let update = reqwest::get(&update_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let toml = toml::from_str::<CargoToml>(&update);

    if toml.is_err() {
        println!("{}", update);
        println!(
            "{} {}",
            "Failed to check for update.".red(),
            toml.err().unwrap()
        );
        return false;
    }

    let toml = toml.unwrap();

    toml.package.version != VERSION
}

pub struct Loader {
    loading: Arc<Mutex<bool>>,
}

impl Loader {
    pub fn new(message: &str) -> Self {
        let loading: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
        let loading_clone = Arc::clone(&loading);
        let message = Arc::new(Mutex::new(message.to_owned()));

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            let mut stdout = io::stdout();
            loop {
                let is_loading = {
                    let guard = loading_clone.lock().unwrap();
                    *guard
                };

                if is_loading {
                    let total_message =
                        loading_str(&format!("[ ] {}", message.lock().unwrap()), None);
                    let message_len = total_message.len();
                    let spaces = terminal_width() - message_len;

                    print!("\r{}{}", total_message, " ".repeat(spaces + 2));
                } else {
                    break;
                }

                interval.tick().await;
                stdout.flush().unwrap();
            }
        });

        Self { loading }
    }

    pub fn stop(&self) {
        let mut guard = self.loading.lock().unwrap();
        *guard = false;
        io::stdout().flush().unwrap();
        print!("\r{}", " ".repeat(terminal_width()));
        print!("\r");
    }
}
