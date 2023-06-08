mod command_utils;
mod git;
mod gpt_api;
mod os_info;
mod query_params;
mod utils;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use colored::Colorize;

use gpt_api::query;

use crate::{
    command_utils::{parse_command, parse_commands},
    git::{build_commands, Git},
    gpt_api::init,
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
            "--force-update, -f:".magenta(),
            "Forces the update to the latest version"
        );
        println!(
            "{} {}",
            "--init, -i [files]:".magenta(),
            "Initializes a README.md file in the current directory based on the content of the given files"
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

    if args.contains(&"--force-update".to_owned()) || args.contains(&"-f".to_owned()) {
        let result = utils::download_update().await;
        match result {
            Ok(_) => println!("{}", "Updated successfully".bright_green()),
            Err(err) => println!("{} {}", "Error:".red(), err),
        }
        return;
    }

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

    let config = &mut utils::get_config();

    if args.contains(&"--api-key".to_owned()) {
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
    } else if config.get_api_key().is_empty() {
        println!("{}", "No API key set. Set API key first!".yellow());
        return;
    }

    if args.contains(&"--clear-api-key".to_owned()) {
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

    if args.contains(&"--init".to_owned()) || args.contains(&"-i".to_owned()) {
        let mut pos = args.iter().position(|s| s == "--init");
        if pos.is_none() {
            pos = args.iter().position(|s| s == "-i");
            if pos.is_none() {
                println!(
                    "{} {}",
                    "Error:".red(),
                    "No files specified to initialize README.md"
                );
                return;
            }
        }
        let pos = pos.unwrap();
        if pos + 1 >= args.len() {
            println!(
                "{} {}",
                "Error:".red(),
                "No files specified to initialize README.md"
            );
            return;
        }
        let files = args[pos + 1..].to_vec();

        let loader = utils::Loader::new("Waiting for response from GPT-3");

        let result = init(&git, files).await;

        loader.stop();

        match result {
            Ok(_) => {
                let path = Path::new("README.md");
                if path.exists() {
                    fs::remove_file(&path).unwrap();
                }
                let mut file = match File::create(&path) {
                    Err(why) => {
                        println!("{} {}", "Error:".red(), why);
                        std::process::exit(1);
                    }
                    Ok(file) => file,
                };
                let write = file.write(result.unwrap().as_bytes());
                if write.is_err() {
                    println!("{} {}", "Error:".red(), write.unwrap_err());
                    std::process::exit(1);
                }
                println!("{}", "README.md initialized successfully".bright_green());
            }
            Err(err) => return println!("{} {}", "Error:".red(), err),
        }

        print!("\n{} {}: ", "Commit".green(), "(Y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("");
        if input.trim() == "y" || input.trim() == "" || input.trim() == "Y" {
            git.add_all(Some(&args));
            let commit_result = git.commit(&"Created README.md".to_owned());

            if commit_result.is_err() {
                println!("{} {}", "Error:".red(), commit_result.unwrap_err());
                std::process::exit(1);
            }

            if push {
                println!("");
                git.push();
            }

            std::process::exit(0);
        } else if input.trim() == "n" || input.trim() == "N" {
            println!("{}", "Aborted".red());
            std::process::exit(0);
        }
        println!("{}", "Invalid input".red());
        std::process::exit(1);
    }

    let loader = utils::Loader::new("Waiting for response from GPT-3");

    let result = query(None, &git, args.clone()).await;

    loader.stop();

    let result = match result {
        Ok(result) => result,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let command = build_commands(&result, push, &args);

    let parsed_command = parse_commands(&command, true);

    println!("{}\n{}", "Commands:".bright_magenta(), parsed_command);
    print!("\n{} {}: ", "Confirm".green(), "(Y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    println!("");
    if input.trim() == "y" || input.trim() == "Y" || input.trim() == "" {
        git.add_all(Some(&args));
        let commit_result = git.commit(&result);

        if commit_result.is_err() {
            println!("{} {}", "Error:".red(), commit_result.unwrap_err());
            std::process::exit(1);
        }

        if push {
            println!("");
            git.push();
        }

        std::process::exit(0);
    } else if input.trim() == "n" || input.trim() == "N" {
        println!("{}", "Aborted".red());
        std::process::exit(0);
    }
    println!("{}", "Invalid input".red());
    std::process::exit(1);
}
