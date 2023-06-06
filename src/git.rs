use colored::Colorize;
use git2::{DiffFormat, DiffOptions, Repository, StatusOptions};
use normpath::{BasePathBuf, PathExt};
use std::{path::Path, str};

pub fn build_commands(commit_message: String, include_push: bool) -> String {
    let mut commit_message = commit_message;

    if commit_message.starts_with("\"") && commit_message.ends_with("\"") {
        commit_message = commit_message[1..commit_message.len() - 1].to_owned();
    }

    let mut commands = String::new();
    commands.push_str("git add .");
    commands.push_str(" && ");
    commands.push_str("git commit -m");
    commands.push(' ');
    commands.push_str(format!("\"{}\"", commit_message).as_str());
    if include_push {
        commands.push_str(" && ");
        commands.push_str("git push");
    }

    commands
}

pub struct Git {
    pub repo: Repository,
    _path: String,
}

impl Git {
    pub fn new(path: String) -> Result<Self, git2::Error> {
        let repo = Repository::open(path.to_owned());
        if repo.is_err() {
            return Err(repo.err().unwrap());
        }
        Ok(Self {
            repo: repo.unwrap(),
            _path: path.to_owned(),
        })
    }

    pub fn clone(self: &Self) -> Git {
        Git::new(self._path.clone()).unwrap()
    }

    pub fn get_diff(self: &Self, files: Option<Vec<String>>) -> Result<String, git2::Error> {
        let repo = &self.repo;
        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;
        let mut patch = String::new();
        let options = &mut DiffOptions::new();
        options.include_untracked(true);
        options.recurse_untracked_dirs(true);
        options.include_unmodified(true);
        options.show_untracked_content(true);
        options.include_typechange(true);
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(options))?;

        let paths: Vec<String> = files.clone().unwrap_or(vec![]);
        let paths: Vec<BasePathBuf> = paths
            .iter()
            .map(|entry| Path::new(entry).normalize())
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap())
            .collect();

        let mut patch_file = String::new();
        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            let file_path = delta.new_file().path().unwrap();

            if paths.is_empty() || paths.contains(&file_path.normalize().unwrap()) {
                let content = str::from_utf8(line.content()).unwrap();
                match line.origin() {
                    '+' => {
                        patch_file.push_str(format!("{}{}", "+".green(), content.green()).as_str())
                    }
                    '-' => patch_file.push_str(format!("{}{}", "-".red(), content.red()).as_str()),
                    'H' => patch_file.push_str(format!("{}", content.bright_blue()).as_str()),
                    'F' => patch_file.push_str(format!("\n{}", content.bold()).as_str()),
                    _ => patch_file.push_str(content),
                }
            }
            true
        })?;
        patch.push_str(&patch_file);
        Ok(patch)
    }

    pub fn get_status(self: &Self) -> Result<String, git2::Error> {
        let repo = &self.repo;
        let options = &mut StatusOptions::new();
        options.include_untracked(true);
        options.include_ignored(false);
        options.recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(options))?;
        let mut status_value = String::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap();
            let status = entry.status();
            let status = match status {
                git2::Status::CURRENT => break,
                git2::Status::INDEX_NEW => "A",
                git2::Status::INDEX_MODIFIED => "M",
                git2::Status::INDEX_DELETED => "D",
                git2::Status::INDEX_RENAMED => "R",
                git2::Status::INDEX_TYPECHANGE => "Typechange",
                git2::Status::WT_NEW => "A",
                git2::Status::WT_MODIFIED => "M",
                git2::Status::WT_DELETED => "D",
                git2::Status::WT_TYPECHANGE => "Typechange",
                git2::Status::WT_RENAMED => "R",
                git2::Status::IGNORED => "!",
                git2::Status::CONFLICTED => "U",
                _ => {
                    if status.is_wt_new() || status.is_index_new() {
                        "A"
                    } else if status.is_wt_modified() || status.is_index_modified() {
                        "M"
                    } else if status.is_wt_deleted() || status.is_index_deleted() {
                        "D"
                    } else if status.is_wt_renamed() || status.is_index_renamed() {
                        "R"
                    } else if status.is_conflicted() {
                        "U"
                    } else {
                        "Unknown"
                    }
                }
            };
            match status {
                "A" => status_value.push_str(format!("{}", path.green()).as_str()),
                "M" => status_value.push_str(format!("{}", path.yellow()).as_str()),
                "D" => status_value.push_str(format!("{}", path.red()).as_str()),
                "R" => status_value.push_str(format!("{}", path.blue()).as_str()),
                "!" => status_value.push_str(format!("{}", path.red()).as_str()),
                "U" => status_value.push_str(format!("{}", path.red()).as_str()),
                &_ => status_value.push_str(path),
            }
            status_value.push_str(format!(" {}", status).as_str());
            status_value.push('\n');
        }

        Ok(status_value)
    }
}
