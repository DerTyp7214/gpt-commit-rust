use std::path::Path;

use normpath::{BasePathBuf, PathExt};

use crate::{git::Git, os_info::get_os_info};

fn get_params() -> Vec<String> {
    let params = vec![
        "You write an informative commit message.",
        "You write a commit subject and body, separated with a new line.",
        "You just reply with 2 lines in total.",
        "Subject line should include One Gitmoji, A Short Summary, using imperative, start with upper case, doesn't end with a period and should not be longer than 50 characters",
        "1. Limit the subject line to 50 characters",
        "2. Use one Gitmoji at the start of the subject line",
        "3. Use imperative in the subject line",
        "4. Wrap the body at 72 characters",
        "5. Use the body to explain what and why vs. how",
        "6. Do not use markdown headings",
    ];

    params.iter().map(|s| s.to_string()).collect()
}

fn get_readme_params() -> Vec<String> {
    let params = vec!["create a readme based on the content of the given info"];

    params.iter().map(|s| s.to_string()).collect()
}

pub fn build_initial_message() -> String {
    let params = get_params().join("\n");
    let os_info = get_os_info();

    format!(
        "# The system information:\n{}\n\n# Your instructions:\n{}",
        os_info, params
    )
}

pub fn build_query(git: &Git, files: Vec<String>) -> String {
    let diff = git.clone().get_diff(Some(files)).unwrap();
    let status = git.clone().get_status().unwrap();

    format!(
        "# Git-Status:\n{}\n\n# Git-Diffs, everything from here is the diff:\n{}",
        status, diff
    )
}

pub fn build_readme_query(git: &Git, files: Vec<String>) -> String {
    let remotes = git.clone().repo.remotes().unwrap();

    let params = get_readme_params().join("\n");
    let origin = remotes.get(0).unwrap();
    let content = get_contents(files);

    let main = format!("{}\n\n{}\n\n", origin, params);

    let content = if content.len() > 4096 - main.len() {
        content[..4096 - main.len()].to_owned()
    } else {
        content
    };

    format!("{}\n\n{}", main, content)
}

fn get_contents(files: Vec<String>) -> String {
    let mut contents = String::new();

    let paths: Vec<BasePathBuf> = files
        .iter()
        .map(|entry| Path::new(entry).normalize())
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .collect();

    for path in paths {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let content = std::fs::read_to_string(path.as_path()).unwrap();
        contents.push_str(format!("## {}\n\n", file_name).as_str());
        contents.push_str(content.as_str());
    }

    contents
}
