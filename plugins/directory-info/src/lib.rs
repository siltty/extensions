//! Directory Info Plugin
//!
//! Shows useful info about the current directory:
//! - Number of files
//! - Project type (Rust/Node/Python/Go)
//! - Directory size

use siltty_plugin_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("directory-info v0.1.0 loaded");
    siltty::set_status_priority(30);
    siltty::set_status_color("#7aa2f7"); // blue
    siltty::set_timer_interval(5000); // update every 5 seconds
}

#[unsafe(no_mangle)]
pub extern "C" fn on_cd() {
    update_info();
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    update_info();
}

fn update_info() {
    let cwd = siltty::get_cwd();
    if cwd.is_empty() {
        return;
    }

    let mut parts: Vec<String> = Vec::new();

    // Detect project type
    if siltty::file_exists(&format!("{cwd}/Cargo.toml")) {
        parts.push("\u{e7a8}".to_string()); // Rust icon
    } else if siltty::file_exists(&format!("{cwd}/package.json")) {
        parts.push("\u{e718}".to_string()); // Node icon
    } else if siltty::file_exists(&format!("{cwd}/go.mod")) {
        parts.push("\u{e626}".to_string()); // Go icon
    } else if siltty::file_exists(&format!("{cwd}/requirements.txt"))
        || siltty::file_exists(&format!("{cwd}/pyproject.toml"))
    {
        parts.push("\u{e73c}".to_string()); // Python icon
    }

    // File count
    if let Some(output) = siltty::run_command("ls -1A") {
        let count = output.lines().count();
        if count > 0 {
            parts.push(format!("{count} files"));
        }
    }

    if parts.is_empty() {
        siltty::set_status_text("");
    } else {
        siltty::set_status_text(&parts.join(" \u{2022} ")); // bullet separator
    }
}
