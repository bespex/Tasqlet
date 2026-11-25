//! The settings in a window of their own, as in Maillet and CodeDeck (D40): room for tabs, and it
//! stays while another window has the focus. While it is open the popover stays open as well, so a
//! change of theme or language shows there at once; closing the settings gives the popover the
//! focus back, so it hides again with the next click elsewhere or Esc.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::i18n::{Key, t};
use crate::popover;

pub const LABEL: &str = "settings";

/// Set before the window is built and cleared when it is gone: the popover loses the focus to the
/// new window possibly before that window counts as visible.
static OPEN: AtomicBool = AtomicBool::new(false);

/// Open the settings, or bring them to the front. Async: building a window in a synchronous
/// command can block the main thread.
#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    open(&app)
}

pub fn open(app: &AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.unminimize();
        let _ = w.show();
        return w.set_focus().map_err(|e| e.to_string());
    }
    OPEN.store(true, Ordering::Relaxed);
    let w = WebviewWindowBuilder::new(
        app,
        LABEL,
        WebviewUrl::App("index.html?view=settings".into()),
    )
    .title(format!("{} – Tasqlet", t(Key::Settings)))
    .inner_size(720.0, 620.0)
    .min_inner_size(560.0, 420.0)
    .center()
    .build()
    .map_err(|e| {
        OPEN.store(false, Ordering::Relaxed);
        e.to_string()
    })?;
    let handle = app.clone();
    w.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            OPEN.store(false, Ordering::Relaxed);
            popover::refocus(&handle);
        }
    });
    Ok(())
}

/// Whether the settings are on screen: then the popover does not hide when it loses the focus.
pub fn is_open(app: &AppHandle) -> bool {
    OPEN.load(Ordering::Relaxed)
        || app
            .get_webview_window(LABEL)
            .is_some_and(|w| w.is_visible().unwrap_or(false))
}
