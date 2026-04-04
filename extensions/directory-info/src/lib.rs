//! Directory Info Plugin v0.2.0
//!
//! Shows project type + file count. Updates on cd + timer fallback.

use siltty_plugin_sdk as siltty;
use std::cell::Cell;

struct State { last_cwd: Cell<u64> }
unsafe impl Sync for State {}
static STATE: State = State { last_cwd: Cell::new(0) };

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u64); }
    h
}

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("directory-info v0.2.0 loaded");
    siltty::set_status_priority(30);
    siltty::set_status_color("#7aa2f7");
    siltty::set_timer_interval(5000);
}

#[unsafe(no_mangle)]
pub extern "C" fn on_cd() { update_info(); }

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    if STATE.last_cwd.get() == 0 {
        let cwd = siltty::get_cwd();
        if !cwd.is_empty() { update_info(); }
    }
}

fn update_info() {
    let cwd = siltty::get_cwd();
    if cwd.is_empty() { return; }

    let hash = simple_hash(&cwd);
    if hash == STATE.last_cwd.get() { return; }
    STATE.last_cwd.set(hash);

    let mut parts: Vec<String> = Vec::new();

    if siltty::file_exists(&format!("{cwd}/Cargo.toml")) {
        parts.push("\u{e7a8}".to_string());
    } else if siltty::file_exists(&format!("{cwd}/package.json")) {
        parts.push("\u{e718}".to_string());
    } else if siltty::file_exists(&format!("{cwd}/go.mod")) {
        parts.push("\u{e626}".to_string());
    } else if siltty::file_exists(&format!("{cwd}/requirements.txt"))
        || siltty::file_exists(&format!("{cwd}/pyproject.toml")) {
        parts.push("\u{e73c}".to_string());
    }

    if let Some(output) = siltty::run_command("ls -1A") {
        let count = output.lines().count();
        if count > 0 { parts.push(format!("{count} files")); }
    }

    if parts.is_empty() {
        siltty::set_status_text("");
    } else {
        siltty::set_status_text(&parts.join(" \u{2022} "));
    }
}
