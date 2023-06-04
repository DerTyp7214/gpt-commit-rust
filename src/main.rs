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
    utils::{check_for_update, get_executable_name},
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let args = std::env::args().collect::<Vec<String>>()[1..].to_vec();

    if args.contains(&"--help".to_owned()) || args.contains(&"-h".to_owned()) {
        let usage_str = format!(
            "{} [optional:option] [optional:files]",
            get_executable_name()
        );
        println!(
            "{} {}\n\n{}",
            "Usage:",
            parse_command(usage_str.as_str(), false),
            "Options:".bright_blue()
        );
        println!("{} {}", "--help, -h:".magenta(), "Shows this help message");
        println!(
            "{} {}",
            "--version, -v:".magenta(),
            "Shows the current version of the program"
        );
        println!(
            "{} {}",
            "--update, -u:".magenta(),
            "Updates the program to the latest version"
        );
        println!(
            "{} {}",
            "--push, -p:".magenta(),
            "Pushes the changes to the remote repository after running the commands"
        );
        println!(
            "{} {}",
            "--api-key:".magenta(),
            "Sets the API key to use for GPT-3. You can also set the API key in the .env file"
        );
        println!(
            "{} {}",
            "--clear-api-key:".magenta(),
            "Clears the API key from the config file"
        );
        return;
    }

    let update_ready = check_for_update().await;

    if update_ready {
        let update_message = parse_command(
            format!("{} --update", get_executable_name()).as_str(),
            false,
        );
        println!(
            "{} Run `{}` to update\n",
            "Update ready".bright_green(),
            update_message
        );
    }

    if args.contains(&"--version".to_owned()) || args.contains(&"-v".to_owned()) {
        println!(
            "{} {}",
            "Version:".bright_magenta(),
            env!("CARGO_PKG_VERSION")
        );
        return;
    }

    if args.contains(&"--update".to_owned()) || args.contains(&"-u".to_owned()) {
        if !update_ready {
            println!("{}", "No update available".yellow());
            return;
        }
        let result = utils::download_update().await;
        match result {
            Ok(_) => println!("{}", "Updated successfully".bright_green()),
            Err(err) => println!("{} {}", "Error:".red(), err),
        }
        return;
    }

    if args.contains(&"--api-key".to_owned()) {
        let config = &mut utils::get_config();
        let pos = args.iter().position(|s| s == "--api-key").unwrap();
        if pos + 1 >= args.len() {
            let api_key = config.get_api_key();
            if api_key.is_empty() {
                println!("{}", "No API key set".yellow());
            } else {
                println!("{}: {}", "API key".bright_magenta(), api_key);
            }
            return;
        }
        let api_key = args[pos + 1].clone();
        config.set_api_key(api_key);
        config.save();

        println!("{}", "API key set".green());
        return;
    }

    if args.contains(&"--clear-api-key".to_owned()) {
        let config = &mut utils::get_config();
        config.set_api_key("".to_owned());
        config.save();

        println!("{}", "API key cleared".green());
        return;
    }

    let mut push = false;
    if args.contains(&"--push".to_owned()) || args.contains(&"-p".to_owned()) {
        push = true;
    }

    let args = args
        .into_iter()
        .filter(|s| s != "--push" && s != "-p")
        .collect::<Vec<String>>();

    let git = Git::new(env::current_dir().unwrap().to_str().unwrap().to_owned());

    if git.is_err() {
        println!("{} {}", "Error:".red(), "Not a git repository");
        return;
    }

    let git = git.unwrap();

    let loader = utils::Loader::new("Waiting for response from GPT-3");

    let result = query(None, git, args).await;

    loader.stop();

    let mut result = match result {
        Ok(result) => build_commands(result, false),
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    if push {
        result.push_str(" && git push");
    }

    let parsed_command = parse_command(&result, true);

    println!("{}\n{}", "Commands:".bright_magenta(), parsed_command);
    print!("\n{} {}: ", "Confirm".green(), "(Y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    println!("");
    if input.trim() == "y" || input.trim() == "Y" || input.trim() == "" {
        run_commands(&result);
        std::process::exit(0);
    } else if input.trim() == "n" || input.trim() == "N" {
        println!("{}", "Aborted".red());
        std::process::exit(0);
    }
    println!("{}", "Invalid input".red());
    std::process::exit(1);
}
