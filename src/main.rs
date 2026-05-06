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

fn list_tracked_repo_files(repo_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let repo = Repository::open(repo_path)?;

    // Use the git index so we copy all tracked files in the repository.
    let index = repo.index()?;
    let files = index
        .iter()
        .filter_map(|entry| {
            let path = std::str::from_utf8(&entry.path).ok()?;
            if path == ".gitignore" {
                return None;
            }
            Some(path.to_string())
        })
        .collect();

    Ok(files)
}

fn handle_copy(path: &str) {
    println!("Copying from: {path}");
    let repo_files = list_tracked_repo_files(path);
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

    copy_files(true, path, &run_dir, repo_files);
}

fn copy_files(create_dir: bool, path: &str, run_dir: &str, repo_files: Result<Vec<String>, Box<dyn std::error::Error>>) {
    match repo_files {
        Ok(files) => {
            println!("Files to copy from {path}:");
            for file in files {
                let src = Path::new(path).join(&file);
                let dst = Path::new(&run_dir).join(&file);

                if create_dir {
                    if let Some(parent) = dst.parent() {
                        if let Err(err) = fs::create_dir_all(parent) {
                            println!("Failed to create parent directory for '{file}': {err}");
                            continue;
                        }
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
    println!("Restoring from target path to git repo: {} {}", git_repo, target_path);

    let target_path_exists = Path::new(target_path).exists();

    if !target_path_exists {
        println!("target_path does not exist: {target_path}");
        std::process::exit(1);
    }

    restore_files_to_git_repo(target_path, git_repo);
}

fn restore_files_to_git_repo(target_path: &str, git_repo: &str) {
    println!("Restoring files from {target_path} to git repo at {git_repo}");

    let repo_files = collect_relative_files(target_path);
    copy_files(false, target_path, git_repo, repo_files);
}


fn collect_relative_files(root: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let root_path = Path::new(root);
    let mut files = Vec::new();
    collect_relative_files_recursive(root_path, root_path, &mut files)?;
    Ok(files)
}

fn collect_relative_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_relative_files_recursive(root, &path, files)?;
            continue;
        }

        if path.is_file() {
            let relative = path.strip_prefix(root)?;
            files.push(relative.to_string_lossy().to_string());
        }
    }

    Ok(())
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

    #[test]
    fn copy_files_from_src_to_dest_copies_single_file() {
        let base = std::env::temp_dir().join(format!(
            "gitcopyrestore_test_{}",
            get_epoch_time()
        ));
        let src_root = base.join("src");
        let dst_root = base.join("dst");
        let rel_file = "nested/file.txt";

        fs::create_dir_all(src_root.join("nested")).expect("create src nested dir");
        fs::create_dir_all(&dst_root).expect("create dst dir");
        fs::write(src_root.join(rel_file), "hello copy").expect("write source file");

        let repo_files: Result<Vec<String>, Box<dyn std::error::Error>> =
            Ok(vec![rel_file.to_string()]);

        copy_files(
            true,
            src_root.to_str().expect("src path utf8"),
            dst_root.to_str().expect("dst path utf8"),
            repo_files,
        );

        let copied = dst_root.join(rel_file);
        assert!(copied.is_file());
        let copied_contents = fs::read_to_string(copied).expect("read copied file");
        assert_eq!(copied_contents, "hello copy");

        let _ = fs::remove_dir_all(base);
    }
}
