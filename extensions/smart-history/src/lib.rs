//! Smart History Plugin
//! 
//! Tests ALL plugin API capabilities:
//! - Status bar with colors and priority
//! - Persistent storage (command count survives restart)
//! - Config (customizable display)
//! - Notifications (on error commands)
//! - Screen reading
//! - Timer with custom interval
//! - CWD change detection
//! - Exec (file counting)

use siltty_ext_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("smart-history v0.1.0 loaded");
    siltty::set_status_priority(60);
    siltty::set_status_color("#7aa2f7"); // blue
    
    // Custom timer: check every 5 seconds
    siltty::set_timer_interval(5000);
    
    // Load persistent command count
    let count = siltty::storage_get("total_commands")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    
    siltty::log(&format!("Loaded history: {} commands tracked", count));
    
    // Show initial status
    update_display(count, None);
}

#[unsafe(no_mangle)]
pub extern "C" fn on_prompt() {
    // Increment command counter
    let count = siltty::storage_get("total_commands")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
        + 1;
    
    siltty::storage_set("total_commands", &count.to_string());
    
    update_display(count, None);
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    let exit_code = siltty::get_last_exit();
    let cmd = siltty::get_last_command();
    
    // Track error count
    if exit_code != 0 {
        let errors = siltty::storage_get("error_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
            + 1;
        siltty::storage_set("error_count", &errors.to_string());
        
        // Store last error
        let error_info = format!("{} (exit {})", cmd, exit_code);
        siltty::storage_set("last_error", &error_info);
        
        // Check if user wants notifications on errors
        let notify_errors = siltty::config_get("notify_on_error")
            .map(|v| v == "true")
            .unwrap_or(true); // default: yes
        
        if notify_errors && !cmd.is_empty() {
            let cmd_short = cmd.split_whitespace().next().unwrap_or("?");
            siltty::send_notification(
                &format!("Command failed: {}", cmd_short),
                &format!("{} exited with code {}", cmd, exit_code),
            );
        }
        
        // Show error in status bar
        siltty::set_status_color("#f7768e"); // red
        let count = siltty::storage_get("total_commands")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        update_display(count, Some(exit_code));
    } else {
        siltty::set_status_color("#7aa2f7"); // blue
        let count = siltty::storage_get("total_commands")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        update_display(count, None);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn on_cd() {
    // Read screen content to verify screen reading works
    let screen = siltty::get_screen_content();
    let lines = screen.lines().count();
    siltty::log(&format!("Screen has {} lines", lines));
    
    // Count files in new directory
    let file_count = siltty::run_command("wc -l")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    if !file_count.is_empty() {
        siltty::log(&format!("Directory listing: {}", file_count));
    }
    
    let count = siltty::storage_get("total_commands")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    update_display(count, None);
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    // Periodic: just ensure status is up to date
    let count = siltty::storage_get("total_commands")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    update_display(count, None);
}

fn update_display(count: u32, last_exit: Option<i32>) {
    // Check config for display format
    let format = siltty::config_get("format")
        .unwrap_or_else(|| "full".to_string());
    
    let errors = siltty::storage_get("error_count")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    
    let text = match format.as_str() {
        "minimal" => {
            format!("#{count}")
        }
        "compact" => {
            if let Some(code) = last_exit {
                format!("#{count} ✗{code}")
            } else {
                format!("#{count}")
            }
        }
        _ => {
            // full format
            let mut parts = vec![format!("#{count}")];
            if errors > 0 {
                parts.push(format!("{errors}err"));
            }
            if let Some(code) = last_exit {
                parts.push(format!("✗{code}"));
            }
            parts.join(" ")
        }
    };
    
    siltty::set_status_text(&text);
}
