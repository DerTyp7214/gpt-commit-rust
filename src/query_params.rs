use crate::{git::Git, os_info::get_os_info};

fn get_params() -> Vec<String> {
    let params = vec![
        "give me a description of these changes if possible in less than 50 characters using gitmoji",
        "dont be like \"added files\" or \"fixed stuff\"",
    ];

    params.iter().map(|s| s.to_string()).collect()
}

pub fn build_query(git: Git, files: Vec<String>) -> String {
    let params = get_params().join("\n");
    let os_info = get_os_info();
    let diff = git.get_diff(Some(files)).unwrap();
    let status = git.get_status().unwrap();

    let main = format!("{}\n\n{}\n\n", os_info, params);

    let diff = if diff.len() > 4096 - main.len() - status.len() {
        diff[..4096 - main.len() - status.len()].to_owned()
    } else {
        diff
    };

    format!("{}\n\n{}\n\n{}", main, diff, status)
}
