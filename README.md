Git copy and restore (gitcr) is a Rust command-line tool that allows you to copy "changed" files from a local git repo.  They are copied to a runs directory with a timestamp and the name of the git repo. Files can be restored to a git repository from the "copy" in the runs directory.

The tool is designed to handle nested directories.


    % cargo test  
    Compiling stable_deref_trait v1.2.1
    [snip]

    running 7 tests
    test tests::checks_arg_count_and_make_sure_copy_or_restore ... ok
    test tests::check_there_are_three_args_for_a_restore ... ok
    test tests::reject_restore_with_only_two_args ... ok
    test tests::rejects_wrong_arg_count ... ok
    test tests::restore_files_to_git_repo_copies_nested_file_when_dest_parent_missing ... ok
    test tests::copy_files_from_src_to_dest_copies_single_file ... ok
    test tests::list_tracked_repo_files_returns_only_local_changes ... ok
    
    test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

    % cargo build --release
    Finished `release` profile [optimized] target(s) in 0.16s

    % cd target/release

    % ./gitcr
    Expected 2 or 3 command-line arguments.
    Usage: gitcr --copy path_to_repo | --restore copy_path path_to_git_repo

    ./gitcr --copy rust/gitcopyrestore 
    Copying from: /rust/gitcopyrestore
    Files to copy from rust/gitcopyrestore:
    Copied: README.md

    % ls runs/1778147782_gitcopyrestore 
    README.md

    % ./gitcr --restore runs/1778147782_gitcopyrestore ./rust/gitcopyrestore 
    Restoring from target path to git repo: runs/1778147782_gitcopyrestore ./rust/gitcopyrestore
    Restoring files from runs/1778147782_gitcopyrestore to git repo at ./rust/gitcopyrestore
    Files to copy from runs/1778147782_gitcopyrestore:
    Copied: README.md