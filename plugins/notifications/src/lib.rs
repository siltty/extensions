use siltty_plugin_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("notifications plugin loaded");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command_finish() {
    let exit_code = siltty::get_last_exit();
    let cmd = siltty::get_last_command();
    if exit_code == 0 {
        siltty::set_status_text(&format!("\u{2713} {cmd}"));
    } else {
        siltty::set_status_text(&format!("\u{2717} {cmd} (exit {exit_code})"));
    }
}
