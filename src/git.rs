use colored::Colorize;
use git2::{DiffFormat, DiffOptions, Oid, Repository, StatusOptions};
use normpath::{BasePathBuf, PathExt};
use std::{path::Path, str};

use crate::command_utils::run_commands;

pub fn build_commands(
    commit_message: &String,
    include_push: bool,
    files: &Vec<String>,
) -> Vec<Vec<String>> {
    let mut commit_message = commit_message.clone();

    if commit_message.starts_with("\"") && commit_message.ends_with("\"") {
        commit_message = commit_message[1..commit_message.len() - 1].to_owned();
    }

    let mut commands: Vec<Vec<String>> = Vec::new();
    let mut add_command: Vec<String> = "git add"
        .to_owned()
        .split(' ')
        .map(|s| s.to_owned())
        .collect();
    add_command.extend(paths_to_git_paths(&files));
    commands.push(add_command);
    let mut commit_command: Vec<String> = "git commit -m"
        .to_owned()
        .split(' ')
        .map(|s| s.to_owned())
        .collect();
    commit_command.push(commit_message);
    commands.push(commit_command);
    if include_push {
        commands.push("git push".split(' ').map(|s| s.to_owned()).collect());
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

    pub fn add_all(self: &Self, files: Option<&Vec<String>>) {
        self.repo
            .index()
            .unwrap()
            .add_all(
                paths_to_git_paths(&files.unwrap_or(&vec![])).iter(),
                git2::IndexAddOption::DEFAULT,
                None,
            )
            .unwrap();
    }

    pub fn commit(
        self: &Self,
        message: &String,
        files: Option<&Vec<String>>,
    ) -> Result<Oid, git2::Error> {
        let mut index = self.repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let signature = self.repo.signature().unwrap();
        let parent_commit = self.repo.head().unwrap().peel_to_commit().unwrap();
        let tree = self.repo.find_tree(oid).unwrap();

        let mut diff_options = git2::DiffOptions::new();
        diff_options.include_ignored(false);
        diff_options.include_untracked(true);
        diff_options.include_unmodified(true);
        diff_options.include_typechange(true);
        diff_options.recurse_untracked_dirs(true);

        let (files_changed, insertions, deletions) = get_file_diff_stats(&self.repo, files);

        let commit_response: Result<Oid, git2::Error> = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&parent_commit],
        );

        match commit_response {
            Ok(commit) => {
                let commit_hash = commit.to_string();
                let commit_hash = commit_hash[..7].to_string();
                let commit_message = message.trim();
                let commit_message = commit_message.replace("\n", " ");
                let commit_message = commit_message.replace("\r", " ");
                let commit_message = commit_message.replace("\t", " ");
                let commit_message = commit_message.replace("  ", " ");

                println!(
                    "[{} {}] {}",
                    self.repo.head().unwrap().shorthand().unwrap(),
                    commit_hash,
                    commit_message
                );
                println!(
                    "{} files changed, {} insertions(+), {} deletions(-)",
                    files_changed, insertions, deletions
                );
            }
            Err(err) => {
                println!("Error: {}", err.message());
                return Err(err);
            }
        };

        commit_response
    }

    pub fn push(self: &Self) {
        run_commands(&vec![vec!["git push".to_owned()]]);
    }
}

fn paths_to_git_paths(paths: &Vec<String>) -> Vec<String> {
    if paths.is_empty() {
        return vec![".".to_owned()];
    }
    paths
        .iter()
        .map(|entry| Path::new(entry))
        .map(|entry| {
            entry
                .into_iter()
                .map(|entry| entry.to_str().unwrap().to_owned())
                .filter(|entry| entry != &".")
                .collect::<Vec<String>>()
                .join("/")
        })
        .collect()
}

fn get_file_diff_stats(
    repo: &Repository,
    file_paths: Option<&Vec<String>>,
) -> (usize, usize, usize) {
    let mut diff_options = DiffOptions::new();
    diff_options.include_unmodified(true);

    let head = repo.head().unwrap();
    let head_commit = head.peel_to_commit().unwrap();
    let tree = head_commit.tree().unwrap();
    let index = repo.index().unwrap();

    let file_paths = file_paths
        .unwrap_or(&vec![])
        .iter()
        .map(|path| {
            Path::new(path)
                .normalize()
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<String>>();

    let diff = repo
        .diff_tree_to_index(Some(&tree), Some(&index), Some(&mut diff_options))
        .unwrap();

    let stats = diff
        .deltas()
        .filter(|delta| {
            file_paths.is_empty()
                || file_paths.contains(
                    &delta
                        .new_file()
                        .path()
                        .unwrap()
                        .normalize()
                        .unwrap()
                        .as_path()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                )
        })
        .map(|delta| {
            let new_file = delta.new_file();
            let old_file = delta.old_file();
            let new_commit = repo.find_commit(new_file.id());
            let old_commit = repo.find_commit(old_file.id());

            println!("{:?} {:?}", new_commit, old_commit);

            if new_commit.is_err() || old_commit.is_err() {
                return None;
            }

            let new_commit = new_commit.unwrap();
            let old_commit = old_commit.unwrap();

            let diff = repo
                .diff_tree_to_tree(
                    Some(&old_commit.tree().unwrap()),
                    Some(&new_commit.tree().unwrap()),
                    None,
                )
                .unwrap();
            Some(diff.stats().unwrap())
        })
        .filter(|stats| stats.is_some())
        .map(|stats| stats.unwrap())
        .fold((0, 0, 0), |acc, status| {
            (
                acc.0 + status.insertions(),
                acc.1 + status.deletions(),
                acc.2 + 1,
            )
        });

    let insertions = stats.0;
    let deletions = stats.1;
    let files_changed = stats.2;

    (files_changed, insertions, deletions)
}
