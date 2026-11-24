//! The popover window: prepared hidden at startup, then only shown and hidden, never recreated
//! (docs/KONZEPT.md 1.1, "Instant und leicht"). Showing emits `popover-shown`, so the frontend can
//! refresh its data.

use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const LABEL: &str = "popover";

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit_to(LABEL, "popover-shown", ());
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
}

#[tauri::command]
pub fn hide_popover(app: AppHandle) {
    hide(&app);
}
