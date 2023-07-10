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
    vec,
};

use colored::Colorize;

use gpt_api::query;
use inquire::{Select, Text};

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
    let files = args
        .iter()
        .filter(|arg| !arg.starts_with("-"))
        .map(|arg| arg.to_owned())
        .collect::<Vec<String>>();

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
            "--no-ai, -n:".magenta(),
            "Commits the changes without using GPT-3"
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

        run(
            &vec!["README.md".to_owned()],
            "Created README.md".to_owned(),
            push,
            &git,
        );
        std::process::exit(0);
    }

    if args.contains(&"--no-ai".to_owned()) || args.contains(&"-n".to_owned()) {
        let result = vec!["#Title", "##Body"].join("\n");
        run(&files, result, push, &git);
        return;
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

    run(&files, result, push, &git);
}

fn run(files: &Vec<String>, result: String, push: bool, git: &Git) {
    let prefixes_to_remove = vec!["Title:", "Body:", "Summary:", "Gitmoji:", "feat:"];

    let result = result
        .split("\n")
        .map(|s| {
            let mut s = s.to_owned();
            for prefix in &prefixes_to_remove {
                s = s.replace(prefix, "").trim().to_owned();
            }
            s.to_owned()
        })
        .collect::<Vec<String>>()
        .join("\n");

    let command = build_commands(&result, push, &files);

    let parsed_command = parse_commands(&command, true);

    println!("{}\n{}\n", "Commands:".bright_magenta(), parsed_command);

    let mut prompt = Select::new("Action", vec!["Run", "Edit", "Abort"]);
    prompt.starting_cursor = 0;
    let prompt = prompt.prompt();
    if prompt.is_err() {
        print!("\n{} {}: ", "Confirm".green(), "(Y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("");
        if input.trim() == "y" || input.trim() == "Y" || input.trim() == "" {
            git.add_old(Some(&files));
            git.commit_old(&result);

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

    let prompt = prompt.unwrap();

    match prompt {
        "Run" => {
            git.add_old(Some(&files));
            git.commit_old(&result);

            if push {
                println!("");
                git.push();
            }

            std::process::exit(0);
        }
        "Edit" => {
            let result = edit(result);
            run(&files, result, push, git);
        }
        "Abort" => {
            println!("{}", "Aborted".red());
            std::process::exit(0);
        }
        _ => {
            println!("{}", "Invalid input".red());
            std::process::exit(1);
        }
    }
}

fn edit(result: String) -> String {
    let mut lines = result
        .split("\n")
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>();
    println!(
        "\n{}\n{}\n",
        "Options:".magenta(),
        &lines
            .iter()
            .enumerate()
            .map(|(index, line)| { format!("{}: {}", (index + 1).to_string().yellow(), line) })
            .collect::<Vec<String>>()
            .join("\n")
    );
    let mut options = vec!["Abort".to_owned()];
    options.extend(
        (1..=lines.len())
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
    );
    let prompt = Select::new("Line to edit", options).prompt();
    if prompt.is_err() {
        println!("{}", "Aborted".red());
        std::process::exit(0);
    }
    let prompt = prompt.unwrap();

    if prompt == "Abort" {
        println!("{}", "Aborted".red());
        std::process::exit(0);
    }

    let prompt = prompt.parse::<usize>();
    if prompt.is_err() {
        println!("{}", "Invalid input".red());
        std::process::exit(1);
    }
    let index = prompt.unwrap() - 1;
    let line_to_edit = lines[index].to_owned();
    let mut prompt = Text::new("Edit");
    prompt.initial_value = Some(&line_to_edit);

    let prompt = prompt.prompt();

    if prompt.is_err() {
        println!("{}", "Aborted".red());
        std::process::exit(0);
    }

    let prompt = prompt.unwrap();

    lines[index] = prompt.as_str();

    lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect::<Vec<&str>>()
        .join("\n")
}
