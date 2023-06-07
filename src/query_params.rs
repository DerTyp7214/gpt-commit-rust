use std::path::Path;

use normpath::{BasePathBuf, PathExt};

use crate::{git::Git, os_info::get_os_info};

fn get_params() -> Vec<String> {
    let params = vec![
        "give me a description of these changes if possible in less than 50 characters using gitmoji",
        "dont be like \"added files\" or \"fixed stuff\"",
    ];

    params.iter().map(|s| s.to_string()).collect()
}

fn get_readme_params() -> Vec<String> {
    let params = vec!["create a readme based on the content of the given info"];

    params.iter().map(|s| s.to_string()).collect()
}

pub fn build_query(git: &Git, files: Vec<String>) -> String {
    let params = get_params().join("\n");
    let os_info = get_os_info();
    let diff = git.clone().get_diff(Some(files)).unwrap();
    let status = git.clone().get_status().unwrap();

    let main = format!("{}\n\n{}\n\n", os_info, params);

    let diff = if diff.len() > 4096 - main.len() - status.len() {
        diff[..4096 - main.len() - status.len()].to_owned()
    } else {
        diff
    };

    format!("{}\n\n{}\n\n{}", main, diff, status)
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
