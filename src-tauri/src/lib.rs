//! Desktop shell of Tasqlet: window, tray and commands. Domain logic belongs in `tasqlet-core`.
//!
//! Plan step 0.2: the popover is prepared hidden and only shown and hidden. Until the tray exists
//! (step 1.4) it is shown once at startup; Esc hides it.

mod popover;

/// Data format version, handed to the frontend so both sides agree on it.
#[tauri::command]
fn schema_version() -> u32 {
    tasqlet_core::SCHEMA_VERSION
}

/// All open items and the global state. Storage follows in plan step 1.2; until then it is empty.
#[tauri::command]
fn snapshot() -> serde_json::Value {
    serde_json::json!({ "schema": tasqlet_core::SCHEMA_VERSION, "on_break": false, "items": [] })
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            schema_version,
            snapshot,
            popover::hide_popover
        ])
        .setup(|app| {
            popover::show(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing only hides: the app lives on in the background (tray from step 1.4).
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == popover::LABEL
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Tasqlet");
}
