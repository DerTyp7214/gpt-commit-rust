use std::{
    fs::File,
    io,
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

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

impl Config {
    pub fn save(&self) {
        let config = toml::to_string(self).unwrap();
        let mut file = File::create("gpt-commit-rust-config.toml").unwrap();
        file.write_all(config.as_bytes()).unwrap();
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.to_owned().unwrap()
    }
}

pub fn get_config() -> Config {
    let config =
        toml::from_str::<Config>(&std::fs::read_to_string("gpt-commit-rust-config.toml").unwrap_or_else(|_| {
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
