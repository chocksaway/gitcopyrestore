fn main() {
    const USAGE: &str = "Usage: gitcopyrestore --copy | --restore path_to_git_repo";
    let args: Vec<String> = std::env::args().skip(1).collect();

    if !has_exactly_two_args_and_correct_copy_or_restore(&args) {
        println!("Expected exactly 2 command-line arguments.");
        println!("{USAGE}");
        return;
    }

    match parse_add_args(&args) {
        Ok((a, b)) => println!("{} + {} = {}", a, b, add(a, b)),
        Err(msg) => {
            println!("{msg}");
            println!("{USAGE}");
        }
    }
}

fn has_exactly_two_args_and_correct_copy_or_restore(args: &[String]) -> bool {
    if args.len() != 2 {
        return false;
    }

    if args[0] != "--copy" && args[0] != "--restore" {
        return false;
    }

    true
}

fn parse_add_args(args: &[String]) -> Result<(i32, i32), &'static str> {
    if args.len() != 2 {
        return Err("Expected exactly 2 command-line arguments.");
    }

    let a = args[0]
        .parse::<i32>()
        .map_err(|_| "First argument must be a valid i32 integer.")?;
    let b = args[1]
        .parse::<i32>()
        .map_err(|_| "Second argument must be a valid i32 integer.")?;

    Ok((a, b))
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*; // import items from parent module

    #[test]
    fn checks_arg_count_and_make_sure_copy_or_restore() {
        let args = vec!["--copy".to_string(), "path".to_string()];
        let args_val = has_exactly_two_args_and_correct_copy_or_restore(&args);
        assert!(args_val);
    }

    #[test]
    fn rejects_wrong_arg_count() {
        let args = vec!["2".to_string()];
        assert!(!has_exactly_two_args_and_correct_copy_or_restore(&args));
    }

    #[test]
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn parses_two_integer_args() {
        let args = vec!["2".to_string(), "3".to_string()];
        assert_eq!(parse_add_args(&args), Ok((2, 3)));
    }

    #[test]
    fn rejects_when_arg_count_is_wrong() {
        let args = vec!["2".to_string()];
        assert!(parse_add_args(&args).is_err());
    }

    #[test]
    fn rejects_non_integer_args() {
        let args = vec!["two".to_string(), "3".to_string()];
        assert!(parse_add_args(&args).is_err());
    }
}

