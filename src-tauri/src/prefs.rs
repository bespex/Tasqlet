//! Settings dialog (plan 2.6 in a slim form): language, popover position, autostart and the learned
//! Jira projects; the global key is in `hotkey`. The rules and the files are the core's; this is the bridge.

use serde_json::{Value, json};
use tasqlet_core::settings::{Language, PopoverPosition, Settings};
use tauri::{AppHandle, Manager};

use crate::data::Data;
use crate::{autostart, popover};

/// Everything the dialog shows.
#[tauri::command]
pub fn get_settings(app: AppHandle) -> Value {
    let data = app.state::<Data>();
    let settings = data.settings();
    let mut value = json!({
        "language": settings.language,
        "popover_position": settings.popover_position,
        "autostart": autostart::enabled(),
        "wayland": !crate::platform::is_x11(),
        "jira": data.jira().projects().map(|(project, base)| json!({"project": project, "base": base})).collect::<Vec<_>>(),
    });
    if let (Value::Object(map), Value::Object(key)) = (&mut value, crate::hotkey::status(&app)) {
        map.extend(key);
    }
    value
}

/// Save language and popover position.
#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    language: Language,
    popover_position: PopoverPosition,
) -> Result<(), String> {
    let data = app.state::<Data>();
    data.set_settings(Settings {
        language,
        popover_position,
        ..data.settings()
    })?;
    popover::set_center(&app, popover_position == PopoverPosition::Center);
    Ok(())
}

/// Switch autostart at login on or off.
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart::set(enabled)
}

/// Forget a learned Jira project (D26).
#[tauri::command]
pub fn forget_jira(app: AppHandle, project: String) -> Result<(), String> {
    app.state::<Data>().forget_jira(&project).map(|_| ())
}
