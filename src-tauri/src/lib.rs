//! Desktop shell of Tasqlet: tray, popover window, single instance and the bridge to the core.
//! Domain logic belongs in `tasqlet-core`.
//!
//! Start-up: a second start hands its request to the running instance and exits (`tasqlet` shows
//! the popover, `tasqlet --toggle` shows or hides it, `tasqlet add …` and the other commands of the
//! command line run there). The first start loads the items, puts the icon into the tray and
//! prepares the popover hidden; a command without a running app works on the files and exits.

mod act;
mod autostart;
mod capture;
mod cli;
mod data;
mod hotkey;
mod i18n;
mod instance;
mod open;
mod platform;
mod popover;
mod prefs;
mod search;
mod settings_window;
mod tray;

use instance::{Request, Start};
use serde_json::Value;
use tasqlet_core::cli::Parsed;
use tauri::{AppHandle, Manager, RunEvent};

/// The argument a desktop binds to a key: show or hide the popover (D10).
pub const TOGGLE: &str = "--toggle";

/// Data format version, handed to the frontend so both sides agree on it.
#[tauri::command]
fn schema_version() -> u32 {
    tasqlet_core::SCHEMA_VERSION
}

/// The open items and the global state, plus what the interface must point out (load error, no
/// tray); `include`: a done item the interface shows anyway (the elaboration of a search hit).
#[tauri::command]
fn snapshot(app: AppHandle, include: Option<String>) -> Value {
    let include = include.and_then(|id| id.parse().ok());
    let mut snapshot = app.state::<data::Data>().snapshot(include);
    snapshot["tray_missing"] = tray::missing(&app).into();
    snapshot
}

pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let request = match tasqlet_core::cli::parse_args(&args) {
        Ok(Parsed::App { toggle: true }) => Request::Toggle,
        Ok(Parsed::App { toggle: false }) => Request::Show,
        Ok(Parsed::Run(command)) => Request::Run { command },
        Ok(Parsed::Help) => {
            print!("{}", tasqlet_core::cli::USAGE);
            return;
        }
        Ok(Parsed::Version) => {
            println!("tasqlet {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Err(e) => {
            eprintln!("tasqlet: {e}\n\n{}", tasqlet_core::cli::USAGE);
            std::process::exit(2);
        }
    };
    let data_dir = data::data_dir();
    let (runtime, key) = (instance::runtime_dir(), instance::key(&data_dir));
    let claimed = instance::claim(&runtime, &key);
    if let Request::Run { command } = &request {
        // The command line: in the running app, else on the files while holding the lock.
        let answer = match &claimed {
            Ok(Start::Running) => instance::send(&runtime, &key, &request)
                .map_err(|e| format!("the running instance does not answer: {e}"))
                .and_then(|answer| answer),
            _ => cli::run_direct(&data_dir, command),
        };
        match answer {
            Ok(out) => print!("{out}"),
            Err(e) => {
                eprintln!("tasqlet: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let instance = match claimed {
        Ok(Start::First(instance)) => Some(instance),
        Ok(Start::Running) => {
            if let Err(e) = instance::send(&runtime, &key, &request)
                .and_then(|a| a.map_err(std::io::Error::other))
            {
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
            popover::set_pinned,
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
            prefs::set_appearance,
            settings_window::open_settings,
            settings_window::window_ready,
            prefs::set_autostart,
            prefs::set_window_form,
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
            popover::set_form(&handle, settings.window_form, settings.sidebar_side);
            tray::start(&handle);
            hotkey::start(&handle, &toggle_key);
            data::watch(&handle);
            if let Some(instance) = &instance {
                let handle = handle.clone();
                instance.listen(move |request| {
                    let app = handle.clone();
                    // Showing a window waits for the main thread: not on the socket's thread.
                    match request {
                        Request::Run { command } => cli::run_in_app(&app, &command),
                        Request::Show => {
                            let _ = handle.run_on_main_thread(move || popover::show(&app, None));
                            Ok(String::new())
                        }
                        Request::Toggle => {
                            let _ = handle.run_on_main_thread(move || popover::toggle_by_key(&app));
                            Ok(String::new())
                        }
                    }
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
