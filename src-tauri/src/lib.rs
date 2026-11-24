//! Desktop shell of Tasqlet: window, tray and commands. Domain logic belongs in `tasqlet-core`.
//!
//! Plan step 0.1 only sets up the skeleton. The prepared, hidden popover follows in step 0.2,
//! tray, single instance and `--toggle` in step 1.4.

/// Data format version, handed to the frontend so both sides agree on it.
#[tauri::command]
fn schema_version() -> u32 {
    tasqlet_core::SCHEMA_VERSION
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![schema_version])
        .run(tauri::generate_context!())
        .expect("error while running Tasqlet");
}
