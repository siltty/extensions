//! Command Timer Plugin
//!
//! Shows a live timer while a command is running.
//! Timer starts when command begins (on_command_start via prompt end)
//! and stops when the next prompt appears.

use siltty_plugin_sdk as siltty;

static mut IDLE: bool = true;
static mut TICKS: u32 = 0;
static mut PROMPT_SEEN: bool = false;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("command-timer v0.2.0 loaded");
    siltty::set_status_priority(10);
    siltty::set_status_color("#565f89");
    siltty::set_timer_interval(1000);
    siltty::set_status_text("");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    // Prompt appeared → command finished, we're idle
    unsafe {
        if !IDLE && TICKS > 2 {
            // Show final time briefly
            let elapsed = format_time(TICKS);
            siltty::set_status_text(&format!("\u{2713} {elapsed}"));
            siltty::set_status_color("#565f89");
        }
        IDLE = true;
        TICKS = 0;
        PROMPT_SEEN = true;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    unsafe {
        IDLE = true;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    unsafe {
        if IDLE {
            // If we're idle and prompt was seen, the timer has nothing to show.
            // Only start counting when prompt_seen transitions from true to
            // false (meaning a command was submitted).
            if PROMPT_SEEN {
                // We saw a prompt. Next timer tick without on_prompt means
                // a command is now running.
                PROMPT_SEEN = false;
            }
            return;
        }
        // We're running a command
        TICKS += 1;
        if TICKS >= 2 {
            // Only show timer after 2 seconds (short commands don't need it)
            let elapsed = format_time(TICKS);
            siltty::set_status_color("#e0af68");
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
