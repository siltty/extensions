//! Command Timer Plugin
//!
//! Shows a live timer while a command is running.
//! Resets on each new prompt.

use siltty_plugin_sdk as siltty;

static mut RUNNING: bool = false;
static mut TICKS: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("command-timer v0.1.0 loaded");
    siltty::set_status_priority(10); // low priority (rightmost)
    siltty::set_status_color("#565f89"); // dimmed
    siltty::set_timer_interval(1000); // tick every second
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    // Command finished, prompt appeared — stop timer
    unsafe {
        if RUNNING && TICKS > 0 {
            let elapsed = format_time(TICKS);
            siltty::set_status_text(&format!("\u{23f1} {elapsed}"));
            siltty::set_status_color("#565f89"); // dim after done
        } else {
            siltty::set_status_text("");
        }
        RUNNING = false;
        TICKS = 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    // Alias for on_prompt — command ended
    unsafe {
        RUNNING = false;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    unsafe {
        if !RUNNING {
            // Detect if a command is running by checking if there is no recent prompt
            // Heuristic: if on_prompt hasn't been called recently, a command is running
            TICKS += 1;
            if TICKS >= 3 {
                // After 3 seconds without prompt, assume command is running
                RUNNING = true;
            }
        }
        if RUNNING {
            TICKS += 1;
            let elapsed = format_time(TICKS);
            siltty::set_status_color("#e0af68"); // yellow while running
            siltty::set_status_text(&format!("\u{23f1} {elapsed}"));
        }
    }
}

fn format_time(secs: u32) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}
