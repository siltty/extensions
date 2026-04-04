//! Git Status Plugin v0.4.0
//!
//! Shows git branch and dirty status. Only checks when something changes.
//! Timer as fallback for initial load (CWD not available in on_init).

use siltty_plugin_sdk as siltty;
use std::cell::Cell;

// WASM is single-threaded — Cell is safe for global state
struct State {
    last_cwd: Cell<u64>,      // hash of last checked CWD
    in_git_repo: Cell<bool>,
}

// SAFETY: WASM plugins are single-threaded
unsafe impl Sync for State {}

static STATE: State = State {
    last_cwd: Cell::new(0),
    in_git_repo: Cell::new(false),
};

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("git-status v0.4.0 loaded");
    siltty::set_status_priority(100);
    siltty::set_timer_interval(5000); // fallback check
}

#[unsafe(no_mangle)]
pub extern "C" fn on_cd() {
    check_repo();
    if STATE.in_git_repo.get() {
        check_dirty();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    if STATE.in_git_repo.get() {
        check_dirty();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    if STATE.in_git_repo.get() {
        check_dirty();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    // Fallback: initial check when CWD becomes available
    if STATE.last_cwd.get() == 0 {
        let cwd = siltty::get_cwd();
        if !cwd.is_empty() {
            check_repo();
            if STATE.in_git_repo.get() {
                check_dirty();
            }
        }
    }
}

fn check_repo() {
    let cwd = siltty::get_cwd();
    if cwd.is_empty() { return; }

    let hash = simple_hash(&cwd);
    if hash == STATE.last_cwd.get() { return; }
    STATE.last_cwd.set(hash);

    let git_dir = format!("{cwd}/.git");
    if !siltty::file_exists(&git_dir) {
        STATE.in_git_repo.set(false);
        siltty::set_status_text("");
        return;
    }

    STATE.in_git_repo.set(true);

    let branch = match siltty::run_command("git branch --show-current") {
        Some(b) => {
            let b = b.trim().to_string();
            if b.is_empty() { "HEAD".to_string() } else { b }
        }
        None => "?".to_string(),
    };

    siltty::storage_set("branch", &branch);
}

fn check_dirty() {
    let branch = siltty::storage_get("branch").unwrap_or_else(|| "?".to_string());

    let is_dirty = match siltty::run_command("git status --porcelain") {
        Some(output) => !output.trim().is_empty(),
        None => false,
    };

    let dirty_marker = if is_dirty { " \u{25cf}" } else { "" };

    if is_dirty {
        siltty::set_status_color("#f7768e");
    } else {
        siltty::set_status_color("#9ece6a");
    }

    let format = siltty::config_get("format").unwrap_or_default();
    let text = match format.as_str() {
        "compact" => format!("{branch}{dirty_marker}"),
        _ => format!("\u{e0a0} {branch}{dirty_marker}"),
    };

    siltty::set_status_text(&text);
}
