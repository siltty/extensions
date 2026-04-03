use siltty_plugin_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("git-status v0.3.0 loaded");
    siltty::set_timer_interval(2000); // check every 2 seconds
    siltty::set_status_priority(100); // high priority (leftmost)
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    update_status();
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    update_status();
}

#[unsafe(no_mangle)]
pub extern "C" fn on_cd() {
    update_status(); // immediate update on directory change
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    update_status();
}

fn update_status() {
    let cwd = siltty::get_cwd();
    if cwd.is_empty() {
        return;
    }

    // Check if we're in a git repo
    let git_dir = format!("{}/.git", cwd);
    if !siltty::file_exists(&git_dir) {
        siltty::set_status_text("");
        return;
    }

    // Get current branch
    let branch = match siltty::run_command("git branch --show-current") {
        Some(b) => {
            let b = b.trim().to_string();
            if b.is_empty() { "HEAD".to_string() } else { b }
        }
        None => "?".to_string(),
    };

    // Get dirty status
    let is_dirty = match siltty::run_command("git status --porcelain") {
        Some(output) => !output.trim().is_empty(),
        None => false,
    };

    let dirty_marker = if is_dirty { " \u{25cf}" } else { "" }; // ●

    // Set color based on clean/dirty state
    if is_dirty {
        siltty::set_status_color("#f7768e"); // red when dirty
    } else {
        siltty::set_status_color("#9ece6a"); // green when clean
    }

    // Support compact vs full format via config
    let format = siltty::config_get("format").unwrap_or_default();
    let status_text = match format.as_str() {
        "compact" => format!("{branch}{dirty_marker}"),
        _ => format!("\u{e0a0} {branch}{dirty_marker}"), //  git branch icon
    };

    siltty::set_status_text(&status_text);
}
