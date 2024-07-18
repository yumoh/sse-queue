fn get_git_hash() -> Option<String> {
    use std::process::Command;
    let branch = Command::new("git")
                         .arg("rev-parse")
                         .arg("--abbrev-ref")
                         .arg("HEAD")
                         .output();
    if let Ok(branch_output) = branch {
        let branch_string = String::from_utf8_lossy(&branch_output.stdout);
        let commit = Command::new("git")
                             .arg("rev-parse")
                             .arg("--verify")
                             .arg("HEAD")
                             .output();
        if let Ok(commit_output) = commit {
            let commit_string = String::from_utf8_lossy(&commit_output.stdout);
            Some(format!("{}, {}",
                        branch_string.lines().next().unwrap_or(""),
                        commit_string.lines().next().unwrap_or("")))
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    if let Some(git) = get_git_hash() {
        // env::set_var("GIT_HASH", git);
        println!("cargo:rustc-env=GIT_HASH={}",&git);
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }
}