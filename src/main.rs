use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use git2::Repository;

fn main() {
    const USAGE: &str = "Usage: gitcr --copy path_to_repo | --restore path_to_git_repo";

    if !Path::new("runs").exists() {
        if let Err(err) = fs::create_dir("runs") {
            println!("Failed to create runs directory: {err}");
            return;
        }
    }

    // let args: Vec<String> = std::env::args().skip(1).collect();
    let args = vec!["--copy".to_string(), "/home/milesd-9510/workspace/rust/gitcopyrestore".to_string()];
    //let args = vec!["--restore".to_string(), "path".to_string(), "repo".to_string()];

    if !has_two_or_three_args_and_correct_copy_or_restore(&args) {
        println!("Expected 2 or 3 command-line arguments.");
        println!("{USAGE}");
        return;
    }

    match args[0].as_str() {
        "--copy" => {
            let path = &args[1];
            handle_copy(path);
        }
        "--restore" => {
            let git_repo = &args[1];
            let target_path = &args[2];
            handle_restore(git_repo, target_path);
        }
        _ => {
            println!("Unknown command: {}", args[0]);
            println!("{USAGE}");
        }
    }
}

fn list_changed_repo_files(repo_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let repo = Repository::open(repo_path)?;

    // Get the HEAD commit
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;

    // Get the diff between HEAD and working directory
    let diff = repo.diff_tree_to_workdir(Some(&tree), None)?;

    // Collect changed files into a vector
    let mut changed_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                changed_files.push(path.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(changed_files)
}

fn handle_copy(path: &str) {
    println!("Copying from: {path}");
    let repo_files = list_changed_repo_files(path);
    mkdir_and_process_files(path, repo_files);
}

fn mkdir_and_process_files(path: &str, repo_files: Result<Vec<String>, Box<dyn std::error::Error>>) {
    let last_path = find_last_part_of_path(path).unwrap_or("unknown_repo");

    if last_path.is_empty() {
        println!("Could not determine the last part of the path. Using 'unknown_repo' as the directory name.");
    }

    let run_dir = format!("runs/{}{}{}", get_epoch_time(), "_", last_path);
    if let Err(err) = fs::create_dir(&run_dir) {
        println!("Failed to create run directory '{run_dir}': {err}");
        return;
    }

    copy_files_to_run_dir(path, &run_dir, repo_files);
}

fn copy_files_to_run_dir(path: &str, run_dir: &str, repo_files: Result<Vec<String>, Box<dyn std::error::Error>>) {
    match repo_files {
        Ok(files) => {
            println!("Files to copy from {path}:");
            for file in files {
                let src = Path::new(path).join(&file);
                let dst = Path::new(&run_dir).join(&file);

                if let Some(parent) = dst.parent() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        println!("Failed to create parent directory for '{file}': {err}");
                        continue;
                    }
                }

                if !src.is_file() {
                    println!("Skipping non-file or missing path: {file}");
                    continue;
                }

                match fs::copy(&src, &dst) {
                    Ok(_) => println!("Copied: {file}"),
                    Err(err) => println!("Failed to copy '{file}': {err}"),
                }
            }
        }
        Err(err) => {
            println!("Failed to list changed files: {err}");
        }
    }
}

fn find_last_part_of_path(path: &str) -> Option<&str> {
    let last = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str());

    last
}

fn get_epoch_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn handle_restore(git_repo: &str, target_path: &str) {
    println!("Restoring from target path git repo: {} {}", git_repo, target_path);

    // TODO: Implement restore logic
}

fn has_two_or_three_args_and_correct_copy_or_restore(args: &[String]) -> bool {
    if args.len() != 2 && args.len() != 3 {
        return false;
    }

    if args[0] != "--copy" && args[0] != "--restore" {
        return false;
    }

    if args[0] == "--restore" && args.len() == 3 {
        return true;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*; // import items from parent module

    #[test]
    fn checks_arg_count_and_make_sure_copy_or_restore() {
        let args = vec!["--copy".to_string(), "path".to_string()];
        let args_val = has_two_or_three_args_and_correct_copy_or_restore(&args);
        assert!(args_val);
    }

    #[test]
    fn rejects_wrong_arg_count() {
        let args = vec!["2".to_string()];
        assert!(!has_two_or_three_args_and_correct_copy_or_restore(&args));
    }

    #[test]
    fn check_there_are_three_args_for_a_restore() {
        let args = vec!["--restore".to_string(), "path".to_string(), "repo".to_string()];
        assert!(has_two_or_three_args_and_correct_copy_or_restore(&args));
    }
}
