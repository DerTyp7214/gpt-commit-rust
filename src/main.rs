mod command_utils;
mod git;
mod gpt_api;
mod os_info;
mod query_params;
mod utils;

use std::env;

use colored::Colorize;
use gpt_api::query;

use crate::{
    command_utils::{parse_command, run_commands},
    git::{build_commands, Git},
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let args = std::env::args().collect::<Vec<String>>()[1..].to_vec();

    if args.contains(&"--api-key".to_owned()) {
        let config = &mut utils::get_config();
        let api_key = args[args.iter().position(|s| s == "--api-key").unwrap() + 1].clone();
        config.set_api_key(api_key);
        config.save();

        println!("{}", "API key set".green());
        return;
    }

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
        Ok(result) => build_commands(result, false),
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let parsed_command = parse_command(&result, true);

    println!("{}\n{}", "Commands:".bright_magenta(), parsed_command);
    print!("\n{} {}: ", "Confirm".green(), "(Y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    println!("");
    if input.trim() == "y" || input.trim() == "Y" || input.trim() == "" {
        run_commands(&result);
    } else {
        println!("Aborted");
    }
}
