//! The items of the running app: one [`Session`] (store with undo), saved in the background and
//! reloaded when someone else changes the files. Every change notifies the tray and the webview.
//!
//! If the data cannot be loaded (damaged, or written by a newer Tasqlet), the app starts empty and
//! **read-only**: nothing is saved, so nothing on disk gets overwritten. The snapshot carries the
//! problem; if a previous version exists, the interface offers to restore it ([`restore_backup`]).

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use chrono::Utc;
use serde_json::{Value, json};
use tasqlet_core::storage::{self, LoadError, Saver, Storage, Watcher};
use tasqlet_core::{Action, Error, SCHEMA_VERSION, Session, Store};
use tauri::{AppHandle, Emitter, Manager};

/// Event to the webview after every change of the items.
pub const CHANGED: &str = "store-changed";

pub struct Data {
    session: Mutex<Session>,
    storage: Storage,
    /// `None` while read-only.
    saver: Mutex<Option<Saver>>,
    problem: Mutex<Option<Problem>>,
    watcher: Mutex<Option<Watcher>>,
}

/// Why the data could not be loaded.
struct Problem {
    message: String,
    /// The damaged file, if it has a backup to restore.
    restorable: Option<PathBuf>,
}

/// The data folder: `$TASQLET_DATA_DIR` or `~/.local/share/tasqlet`.
pub fn data_dir() -> PathBuf {
    storage::default_dir().unwrap_or_else(|| PathBuf::from(".tasqlet"))
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn new_saver(storage: &Storage) -> Saver {
    Saver::new(storage.clone(), |e| {
        eprintln!("tasqlet: saving failed: {e}")
    })
}

impl Data {
    /// Load the items from `dir`.
    pub fn open(dir: PathBuf) -> std::io::Result<Self> {
        let storage = Storage::open(dir)?;
        let (store, saver, problem) = match storage.load() {
            Ok(loaded) => (loaded.store, Some(new_saver(&storage)), None),
            Err(e) => {
                eprintln!("tasqlet: {e}; starting read-only");
                let restorable = match &e {
                    LoadError::Corrupt {
                        path,
                        backup: Some(_),
                        ..
                    } => Some(path.clone()),
                    _ => None,
                };
                let problem = Problem {
                    message: e.to_string(),
                    restorable,
                };
                (Store::new(), None, Some(problem))
            }
        };
        Ok(Self {
            session: Mutex::new(Session::new(store)),
            storage,
            saver: Mutex::new(saver),
            problem: Mutex::new(problem),
            watcher: Mutex::new(None),
        })
    }

    fn session(&self) -> MutexGuard<'_, Session> {
        lock(&self.session)
    }

    /// A copy of the current store.
    pub fn store(&self) -> Store {
        self.session().store().clone()
    }

    /// The snapshot the webview renders: the data file's shape, each resource with the label to
    /// show (`shown`, own or smart label from the core), plus the load problem.
    pub fn snapshot(&self) -> Value {
        let session = self.session();
        let store = session.store();
        let items: Vec<Value> = store
            .items()
            .iter()
            .map(|it| {
                let mut value = serde_json::to_value(it).unwrap_or(Value::Null);
                if let Some(resources) = value["resources"].as_array_mut() {
                    for (json, resource) in resources.iter_mut().zip(&it.resources) {
                        json["shown"] = resource.shown_label().into();
                    }
                }
                value
            })
            .collect();
        let problem = lock(&self.problem);
        json!({
            "schema": SCHEMA_VERSION,
            "on_break": store.on_break(),
            "items": items,
            "load_error": problem.as_ref().map(|p| json!({
                "message": p.message,
                "backup": p.restorable.is_some(),
            })),
        })
    }

    /// Hand the current store to the saver, unless read-only.
    fn save(&self, store: &Store) {
        if let Some(saver) = lock(&self.saver).as_ref() {
            saver.save(store.clone());
        }
    }

    /// Write what is pending, e.g. before quitting.
    pub fn flush(&self) {
        if let Some(saver) = lock(&self.saver).as_ref()
            && let Err(e) = saver.flush()
        {
            eprintln!("tasqlet: saving failed: {e}");
        }
    }
}

/// Carry out an action, save it in the background and tell tray and webview.
pub fn apply(app: &AppHandle, action: Action) -> Result<(), Error> {
    let data = app.state::<Data>();
    {
        let mut session = data.session();
        session.apply(action, Utc::now())?;
        data.save(session.store());
    }
    changed(app);
    Ok(())
}

/// Put the previous version of the damaged file in its place, load again and leave read-only mode.
#[tauri::command]
pub fn restore_backup(app: AppHandle) -> Result<(), String> {
    let data = app.state::<Data>();
    let path = lock(&data.problem)
        .as_ref()
        .and_then(|p| p.restorable.clone())
        .ok_or("there is no backup to restore")?;
    data.storage
        .restore_backup(&path)
        .map_err(|e| e.to_string())?;
    let loaded = data.storage.load().map_err(|e| e.to_string())?;
    data.session().replace(loaded.store);
    *lock(&data.saver) = Some(new_saver(&data.storage));
    *lock(&data.problem) = None;
    watch(&app);
    changed(&app);
    Ok(())
}

/// Reload when someone else changed the files (CLI, editor, sync).
pub fn watch(app: &AppHandle) {
    let data = app.state::<Data>();
    if lock(&data.saver).is_none() {
        return; // read-only: the files are left alone entirely
    }
    let handle = app.clone();
    let watcher = data.storage.watch(move |path| {
        let data = handle.state::<Data>();
        match data.storage.load() {
            Ok(loaded) => {
                // What was pending is older than the change from outside.
                if let Some(saver) = lock(&data.saver).as_ref() {
                    saver.discard();
                }
                data.session().replace(loaded.store);
                changed(&handle);
            }
            Err(e) => eprintln!(
                "tasqlet: {} changed but cannot be read: {e}",
                path.display()
            ),
        }
    });
    match watcher {
        Ok(w) => *lock(&data.watcher) = Some(w),
        Err(e) => eprintln!("tasqlet: not watching the data folder: {e}"),
    }
}

/// Tell tray and webview that the items changed.
pub fn changed(app: &AppHandle) {
    crate::tray::refresh(app);
    let _ = app.emit(CHANGED, ());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tasqlet_core::{NewItem, NewResource, ResourceType, Target};

    #[test]
    fn snapshot_carries_the_label_to_show() {
        let dir = tempfile::tempdir().unwrap();
        let data = Data::open(dir.path().to_path_buf()).unwrap();
        let mut item = NewItem::new("Review");
        item.resources = vec![
            NewResource {
                kind: ResourceType::Url,
                target: "https://github.com/example/core/pull/42".into(),
                label: None,
            },
            NewResource {
                kind: ResourceType::Mail,
                target: "~/Mail/x.eml".into(),
                label: Some("Mail: Ops".into()),
            },
        ];
        data.session()
            .apply(
                Action::Add {
                    item,
                    target: Target::Now,
                },
                Utc::now(),
            )
            .unwrap();
        let snapshot = data.snapshot();
        let resources = &snapshot["items"][0]["resources"];
        assert_eq!(resources[0]["shown"], "PR #42");
        assert_eq!(resources[1]["shown"], "Mail: Ops");
        assert_eq!(snapshot["load_error"], Value::Null);
    }

    #[test]
    fn damaged_data_start_read_only_and_offer_the_backup() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::open(dir.path()).unwrap();
        let mut store = Store::new();
        store
            .add(NewItem::new("eins"), Target::Focus, Utc::now())
            .unwrap();
        storage.save(&store).unwrap();
        store
            .add(NewItem::new("zwei"), Target::Focus, Utc::now())
            .unwrap();
        storage.save(&store).unwrap();
        std::fs::write(storage.items_path(), b"{ kaputt").unwrap();

        let data = Data::open(dir.path().to_path_buf()).unwrap();
        assert!(lock(&data.saver).is_none(), "read-only");
        assert_eq!(data.snapshot()["load_error"]["backup"], true);
        assert!(data.store().items().is_empty());
    }
}
