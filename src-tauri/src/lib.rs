//! Desktop shell of Tasqlet: tray, popover window, single instance and the bridge to the core.
//! Domain logic belongs in `tasqlet-core`.
//!
//! Start-up: a second start hands its request to the running instance and exits (`tasqlet` shows
//! the popover, `tasqlet --toggle` shows or hides it). The first start loads the items, puts the
//! icon into the tray and prepares the popover hidden.

mod act;
mod autostart;
mod capture;
mod data;
mod hotkey;
mod i18n;
mod instance;
mod open;
mod platform;
mod popover;
mod prefs;
mod search;
mod tray;

use instance::{Request, Start};
use serde_json::Value;
use tauri::{AppHandle, Manager, RunEvent};

/// The argument a desktop binds to a key: show or hide the popover (D10).
pub const TOGGLE: &str = "--toggle";

/// Data format version, handed to the frontend so both sides agree on it.
#[tauri::command]
fn schema_version() -> u32 {
    tasqlet_core::SCHEMA_VERSION
}

/// All items and the global state, plus what the interface must point out (load error, no tray).
#[tauri::command]
fn snapshot(app: AppHandle) -> Value {
    let mut snapshot = app.state::<data::Data>().snapshot();
    snapshot["tray_missing"] = tray::missing(&app).into();
    snapshot
}

pub fn run() {
    let request = if std::env::args().skip(1).any(|a| a == TOGGLE) {
        Request::Toggle
    } else {
        Request::Show
    };
    let data_dir = data::data_dir();
    let (runtime, key) = (instance::runtime_dir(), instance::key(&data_dir));
    let instance = match instance::claim(&runtime, &key) {
        Ok(Start::First(instance)) => Some(instance),
        Ok(Start::Running) => {
            if let Err(e) = instance::send(&runtime, &key, request) {
                eprintln!("tasqlet: the running instance does not answer: {e}");
                std::process::exit(1);
            }
            return;
        }
        Err(e) => {
            // Without the lock two instances could write the same files; still better than none.
            eprintln!("tasqlet: cannot check for a running instance: {e}");
            None
        }
    };
    let data = match data::Data::open(data_dir) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("tasqlet: cannot use the data folder: {e}");
            std::process::exit(1);
        }
    };
    let settings = data.settings();
    match settings.language {
        tasqlet_core::settings::Language::De => i18n::force(i18n::Lang::De),
        tasqlet_core::settings::Language::En => i18n::force(i18n::Lang::En),
        tasqlet_core::settings::Language::Auto => {}
    }
    let show_at_start = request == Request::Toggle;
    let toggle_key = settings.toggle_key.clone();

    let app = tauri::Builder::default()
        .plugin(hotkey::plugin())
        .manage(data)
        .invoke_handler(tauri::generate_handler![
            schema_version,
            snapshot,
            popover::hide_popover,
            popover::set_wide,
            data::restore_backup,
            capture::parse_quick,
            capture::capture,
            capture::undo,
            act::act,
            open::open_resource,
            open::open_all,
            search::search,
            prefs::get_settings,
            prefs::set_settings,
            prefs::set_autostart,
            prefs::forget_jira,
            hotkey::set_toggle_key,
            hotkey::suspend_toggle_key
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            popover::setup(&handle);
            popover::set_center(
                &handle,
                settings.popover_position == tasqlet_core::settings::PopoverPosition::Center,
            );
            tray::start(&handle);
            hotkey::start(&handle, &toggle_key);
            data::watch(&handle);
            if let Some(instance) = &instance {
                let handle = handle.clone();
                instance.listen(move |request| {
                    let app = handle.clone();
                    // Showing a window waits for the main thread: not on the socket's thread.
                    let _ = handle.run_on_main_thread(move || match request {
                        Request::Show => popover::show(&app, None),
                        Request::Toggle => popover::toggle(&app, None),
                    });
                })?;
            }
            if show_at_start {
                popover::show(&handle, None);
            }
            // Keep the lock and the socket for the life of the app.
            app.manage(InstanceGuard {
                _instance: instance,
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tasqlet");

    app.run(|app, event| match event {
        // The popover only hides; the app lives on in the tray until "Quit".
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        RunEvent::Exit => app.state::<data::Data>().flush(),
        _ => {}
    });
}

/// Holds the single-instance lock until the app ends.
struct InstanceGuard {
    _instance: Option<instance::Instance>,
}
