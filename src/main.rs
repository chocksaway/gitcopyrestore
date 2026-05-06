use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use git2::{Repository, Status, StatusOptions};

fn main() {
    const USAGE: &str = "Usage: gitcr --copy path_to_repo | --restore copy_path path_to_git_repo";

    if !Path::new("runs").exists() {
        if let Err(err) = fs::create_dir("runs") {
            println!("Failed to create runs directory: {err}");
            return;
        }
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    //let args = vec!["--copy".to_string(), "/home/milesd-9510/workspace/rust/gitcopyrestore".to_string()];
    //let args = vec!["--restore".to_string(), "/home/milesd-9510/workspace/rust/gitcopyrestore/runs/1778082154_gitcopyrestore".to_string(), "/home/milesd-9510/workspace/rust/gitcopyrestore".to_string()];

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

    let mut opts = StatusOptions::new();
    opts.include_untracked(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let files = statuses
        .iter()
        .filter_map(|entry| {
            let status = entry.status();
            let changed = status.intersects(
                Status::INDEX_MODIFIED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE
                    | Status::WT_MODIFIED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE
                    | Status::CONFLICTED,
            );

            if !changed {
                return None;
            }

            let path = entry.path()?;
            if path == ".gitignore" {
                return None;
            }

            Some(path.to_string())
        })
        .collect();

    Ok(files)
}

fn handle_copy(path: &str) {
    let canonical_path = match fs::canonicalize(path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => path.to_string(),
    };
    println!("Copying from: {canonical_path}");
    let repo_files = list_tracked_repo_files(path);
    mkdir_and_process_files(path, repo_files);
}

fn mkdir_and_process_files(src_path: &str, files: Result<Vec<String>, Box<dyn std::error::Error>>) {
    let last_path = find_last_part_of_path(src_path).unwrap_or("unknown_repo");

    if last_path.is_empty() {
        println!("Could not determine the last part of the path. Using 'unknown_repo' as the directory name.");
    }

    let dest_path = format!("runs/{}{}{}", get_epoch_time(), "_", last_path);
    if let Err(err) = fs::create_dir(&dest_path) {
        println!("Failed to create run directory '{dest_path}': {err}");
        return;
    }

    copy_files(true, src_path, &dest_path, files);
}

fn copy_files(create_dir: bool, src_path: &str, dest_path: &str, files: Result<Vec<String>, Box<dyn std::error::Error>>) {
    match files {
        Ok(files) => {
            println!("Files to copy from {src_path}:");
            for file in files {
                let src = Path::new(src_path).join(&file);
                let dst = Path::new(&dest_path).join(&file);

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

fn handle_restore(src_path: &str, dest_path: &str) {
    println!("Restoring from target path to git repo: {} {}", src_path, dest_path);

    let target_path_exists = Path::new(dest_path).exists();

    if !target_path_exists {
        println!("target_path does not exist: {dest_path}");
        std::process::exit(1);
    }

    restore_files_to_git_repo(src_path, dest_path);
}

fn restore_files_to_git_repo(src_path: &str, dest_path: &str) {
    println!("Restoring files from {src_path} to git repo at {dest_path}");

    let files = collect_relative_files(src_path);
    copy_files(true, src_path, &dest_path, files);
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
    if args.len() < 2 || args.len() > 3 {
        return false;
    }

    if args[0] != "--copy" && args[0] != "--restore" {
        return false;
    }

    // --copy requires exactly 2 args
    if args[0] == "--copy" && args.len() != 2 {
        return false;
    }

    // --restore requires exactly 3 args
    if args[0] == "--restore" && args.len() != 3 {
        return false;
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
    fn reject_restore_with_only_two_args() {
        let args = vec!["--restore".to_string(), "path".to_string()];
        assert!(!has_two_or_three_args_and_correct_copy_or_restore(&args));
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

    #[test]
    fn restore_files_to_git_repo_copies_nested_file_when_dest_parent_missing() {
        let base = std::env::temp_dir().join(format!(
            "gitcopyrestore_restore_test_{}",
            get_epoch_time()
        ));
        let snapshot_root = base.join("snapshot");
        let repo_root = base.join("repo");
        let rel_file = "nested/file.txt";

        fs::create_dir_all(snapshot_root.join("nested")).expect("create snapshot nested dir");
        fs::create_dir_all(&repo_root).expect("create repo dir");
        fs::write(snapshot_root.join(rel_file), "restored").expect("write snapshot file");

        restore_files_to_git_repo(
            snapshot_root.to_str().expect("snapshot path utf8"),
            repo_root.to_str().expect("repo path utf8"),
        );

        let restored = repo_root.join(rel_file);
        assert!(restored.is_file());
        let restored_contents = fs::read_to_string(restored).expect("read restored file");
        assert_eq!(restored_contents, "restored");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn list_tracked_repo_files_returns_only_local_changes() {
        let base = std::env::temp_dir().join(format!(
            "gitcopyrestore_status_test_{}_{}",
            get_epoch_time(),
            std::process::id()
        ));
        let repo_root = base.join("repo");
        fs::create_dir_all(&repo_root).expect("create repo dir");

        let repo = Repository::init(&repo_root).expect("init repo");

        fs::write(repo_root.join("unchanged.txt"), "same").expect("write unchanged");
        fs::write(repo_root.join("changed.txt"), "before").expect("write changed before");

        let mut index = repo.index().expect("repo index");
        index
            .add_path(Path::new("unchanged.txt"))
            .expect("add unchanged");
        index
            .add_path(Path::new("changed.txt"))
            .expect("add changed");
        index.write().expect("write index");

        let tree_id = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        let sig = git2::Signature::now("gitcr-test", "gitcr@test.local").expect("signature");
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .expect("initial commit");

        fs::write(repo_root.join("changed.txt"), "after").expect("modify tracked file");
        fs::write(repo_root.join("new.txt"), "new").expect("write untracked file");

        // Simulate an untracked run artifact directory that should never be copied.
        let runs_tmp = repo_root.join("runs/tmp_run/src");
        fs::create_dir_all(&runs_tmp).expect("create runs dir");
        fs::write(runs_tmp.join("ignored.txt"), "ignore me").expect("write runs file");

        let files = list_tracked_repo_files(repo_root.to_str().expect("repo path utf8"))
            .expect("list changed files");

        assert!(files.iter().any(|f| f == "changed.txt"));
        assert!(!files.iter().any(|f| f == "new.txt"));
        assert!(!files.iter().any(|f| f == "unchanged.txt"));
        assert!(!files.iter().any(|f| f.starts_with("runs/")));

        let _ = fs::remove_dir_all(base);
    }
}
