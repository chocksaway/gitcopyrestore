fn main() {
    const USAGE: &str = "Usage: gitcr --copy path_to_repo | --restore path_to_git_repo";
    // let args: Vec<String> = std::env::args().skip(1).collect();
    // let args = vec!["--copy".to_string(), "path".to_string()];
    let args = vec!["--restore".to_string(), "path".to_string(), "repo".to_string()];

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

fn handle_copy(path: &str) {
    println!("Copying from: {}", path);
    // TODO: Implement copy logic
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

