//! The global key that shows or hides the popover (D36, X11 part of plan 3.2). Wayland compositors
//! do not let an app grab keys; there `tasqlet --toggle` on a desktop key stays the way (the portal
//! *GlobalShortcuts* follows with 3.2).

use std::sync::Mutex;

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::data::Data;
use crate::popover;

/// Why a key does not work. The interface turns the code into a sentence.
const INVALID: &str = "invalid";
const TAKEN: &str = "taken";
const UNSUPPORTED: &str = "unsupported";

/// The key held now and why the configured one is not held, if it is not.
#[derive(Default)]
struct HotkeyState {
    keys: Mutex<String>,
    problem: Mutex<Option<&'static str>>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// The plugin, with the handler: a press toggles the popover.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || popover::toggle(&handle, None));
            }
        })
        .build()
}

/// A key as the setting spells it; empty means none.
pub fn parse(keys: &str) -> Result<Option<Shortcut>, &'static str> {
    let keys = keys.trim();
    if keys.is_empty() {
        return Ok(None);
    }
    keys.parse::<Shortcut>().map(Some).map_err(|_| INVALID)
}

fn grab(app: &AppHandle, keys: &str) -> Result<(), &'static str> {
    if !crate::platform::is_x11() {
        return if keys.trim().is_empty() {
            Ok(())
        } else {
            Err(UNSUPPORTED)
        };
    }
    match parse(keys)? {
        None => Ok(()),
        Some(shortcut) => app.global_shortcut().register(shortcut).map_err(|_| TAKEN),
    }
}

/// At start: hold the configured key, or note why not.
pub fn start(app: &AppHandle, keys: &str) {
    app.manage(HotkeyState::default());
    let state = app.state::<HotkeyState>();
    *lock(&state.keys) = keys.trim().to_string();
    if let Err(problem) = grab(app, keys) {
        eprintln!("tasqlet: global key {keys} not available: {problem}");
        *lock(&state.problem) = Some(problem);
    }
}

/// What the settings show: the key and, if it is not held, why.
pub fn status(app: &AppHandle) -> Value {
    let state = app.state::<HotkeyState>();
    json!({
        "toggle_key": *lock(&state.keys),
        "toggle_key_problem": *lock(&state.problem),
    })
}

/// A new key (empty: none). If it cannot be held, the old one stays and nothing is saved.
#[tauri::command]
pub fn set_toggle_key(app: AppHandle, keys: String) -> Result<(), String> {
    let keys = keys.trim().to_string();
    parse(&keys)?;
    let state = app.state::<HotkeyState>();
    let old = lock(&state.keys).clone();
    let _ = app.global_shortcut().unregister_all();
    if let Err(problem) = grab(&app, &keys) {
        let _ = grab(&app, &old);
        return Err(problem.into());
    }
    let data = app.state::<Data>();
    let mut settings = data.settings();
    settings.toggle_key = keys.clone();
    if let Err(e) = data.set_settings(settings) {
        let _ = app.global_shortcut().unregister_all();
        let _ = grab(&app, &old);
        return Err(e);
    }
    *lock(&state.keys) = keys;
    *lock(&state.problem) = None;
    Ok(())
}

/// While the settings record a key, the global one must not fire: the popover would hide.
#[tauri::command]
pub fn suspend_toggle_key(app: AppHandle, suspended: bool) {
    if !crate::platform::is_x11() {
        return;
    }
    let _ = app.global_shortcut().unregister_all();
    if !suspended {
        let keys = lock(&app.state::<HotkeyState>().keys).clone();
        let _ = grab(&app, &keys);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_as_the_settings_record_them_parse() {
        assert!(parse("").unwrap().is_none());
        assert!(parse("Ctrl+Alt+KeyT").unwrap().is_some());
        assert!(parse("Super+Shift+F12").unwrap().is_some());
        assert!(parse("Ctrl+Digit1").unwrap().is_some());
        assert_eq!(parse("Ctrl+Alt+").unwrap_err(), INVALID);
        assert_eq!(parse("Ctrl+Nonsense").unwrap_err(), INVALID);
    }
}
