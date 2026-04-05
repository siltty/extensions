use siltty_ext_sdk as siltty;

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    siltty::log("auto-theme plugin loaded");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_timer() {
    // Placeholder: would check macOS appearance and switch theme
    // For now just set a status indicator
    siltty::set_status_text("\u{1f319}");
}
