mod command_utils;
mod git;
mod gpt_api;
mod os_info;
mod query_params;
mod utils;

use std::env;

use colored::Colorize;
use command_utils::run_command;
use gpt_api::query;

use crate::{command_utils::parse_command, git::Git};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let args = std::env::args().collect::<Vec<String>>()[1..].to_vec();

    let git = Git::new(env::current_dir().unwrap().to_str().unwrap().to_owned());

    if git.is_err() {
        println!("{} {}", "Error:".red(), "Not a git repository");
        return;
    }

    let git = git.unwrap();

    let loader = utils::Loader::new("Waiting for response from GPT-3");

    let result = query(None, git, args).await;

    loader.stop();

    let result = match result {
        Ok(result) => parse_command(&result, true),
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    println!("{}", result);
}
