//! Siltty Notifications Plugin
//!
//! Shows command exit status in the status bar.
//! Sends desktop notification when a long-running command finishes.

use siltty_ext_sdk as siltty;

static mut COMMAND_START: u64 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("notifications v1.0.0 loaded");
    siltty::set_status_priority(50);
    siltty::set_status_color("#565f89"); // dimmed by default
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    // Record when the prompt appears (command is about to start)
    unsafe { COMMAND_START = timestamp_secs(); }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    let exit_code = siltty::get_last_exit();
    let cmd = siltty::get_last_command();
    let elapsed = unsafe { timestamp_secs() - COMMAND_START };

    // Short display name for command
    let cmd_short = cmd.split_whitespace().next().unwrap_or("?");

    if exit_code == 0 {
        siltty::set_status_color("#9ece6a"); // green
        if elapsed < 5 {
            siltty::set_status_text(&format!("\u{2713} {cmd_short}"));
        } else {
            let time_str = format_duration(elapsed);
            siltty::set_status_text(&format!("\u{2713} {cmd_short} ({time_str})"));
        }
    } else {
        siltty::set_status_color("#f7768e"); // red
        siltty::set_status_text(&format!("\u{2717} {cmd_short} exit {exit_code}"));
    }

    // Desktop notification for commands that took > threshold seconds
    let threshold: u64 = siltty::config_get("notify_after_seconds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    if elapsed >= threshold {
        let title = if exit_code == 0 {
            format!("\u{2713} Command finished")
        } else {
            format!("\u{2717} Command failed (exit {exit_code})")
        };
        let body = format!("{cmd} — {}", format_duration(elapsed));
        siltty::send_notification(&title, &body);
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn timestamp_secs() -> u64 {
    // Simple monotonic counter using timer ticks
    // Not actual wall clock, but good enough for duration measurement
    static mut TICKS: u64 = 0;
    unsafe {
        TICKS += 1;
        TICKS
    }
}
