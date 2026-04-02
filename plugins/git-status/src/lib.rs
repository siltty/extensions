use siltty_plugin_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("git-status plugin initialized");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    let cwd = siltty::get_cwd();
    if cwd.is_empty() {
        return;
    }

    // Check if .git directory exists
    let git_dir = format!("{}/.git", cwd);
    let has_git = siltty::file_size(&git_dir) >= 0;

    if !has_git {
        siltty::set_status_text("");
        return;
    }

    // Read HEAD to get branch name
    let head_path = format!("{}/.git/HEAD", cwd);
    let head_size = siltty::file_size(&head_path);
    if head_size <= 0 {
        siltty::set_status_text(" git");
        return;
    }

    // For now, just show that we're in a git repo
    // (full HEAD reading requires fs_read_file which we'll add later)
    siltty::set_status_text(" git repo");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    // Re-check git status after each command
    on_prompt();
}
