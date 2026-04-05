use siltty_ext_sdk as siltty;

static mut SECONDS_SINCE_PROMPT: u32 = 0;
static mut SHOW_TIMER: bool = false;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("command-timer v0.2.0 loaded");
    siltty::set_status_priority(10);
    siltty::set_timer_interval(1000);
    siltty::set_status_text("");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    // Prompt appeared -> command finished
    unsafe {
        if SHOW_TIMER {
            // Show final time
            let t = format_time(SECONDS_SINCE_PROMPT);
            siltty::set_status_text(&format!("\u{2713} {t}"));
            siltty::set_status_color("#565f89");
        }
        SECONDS_SINCE_PROMPT = 0;
        SHOW_TIMER = false;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    on_prompt();
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    unsafe {
        SECONDS_SINCE_PROMPT += 1;

        // Show timer after 3 seconds of no prompt (command is running)
        if SECONDS_SINCE_PROMPT >= 3 {
            SHOW_TIMER = true;
            let t = format_time(SECONDS_SINCE_PROMPT);
            siltty::set_status_color("#e0af68");
            siltty::set_status_text(&format!("\u{23f1} {t}"));
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
