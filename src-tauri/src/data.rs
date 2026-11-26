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
use tasqlet_core::jira::{self, JiraProjects};
use tasqlet_core::settings::{SETTINGS_FILE, Settings};
use tasqlet_core::storage::{self, LoadError, Saver, Storage, Watcher};
use tasqlet_core::{Action, Error, Id, SCHEMA_VERSION, Session, State, Store};
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
    /// Learned Jira projects (D26) and where they are kept; no path if the file was unreadable, so
    /// it is not overwritten.
    jira: Mutex<JiraProjects>,
    jira_path: Option<PathBuf>,
    /// The few settings (`settings.json`); no path if the file was unreadable, so it is not overwritten.
    settings: Mutex<Settings>,
    settings_path: Option<PathBuf>,
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
        let path = jira::config_dir().map(|dir| dir.join(jira::JIRA_FILE));
        let (jira, jira_path) = match path.as_deref().map(JiraProjects::load) {
            Some(Ok(projects)) => (projects, path),
            Some(Err(e)) => {
                eprintln!("tasqlet: Jira projects not readable, not learning: {e}");
                (JiraProjects::default(), None)
            }
            None => (JiraProjects::default(), None),
        };
        let path = jira::config_dir().map(|dir| dir.join(SETTINGS_FILE));
        let (settings, settings_path) = match path.as_deref().map(Settings::load) {
            Some(Ok(settings)) => (settings, path),
            Some(Err(e)) => {
                eprintln!("tasqlet: settings not readable, using defaults and not saving: {e}");
                (Settings::default(), None)
            }
            None => (Settings::default(), None),
        };
        Ok(Self {
            session: Mutex::new(Session::new(store)),
            storage,
            saver: Mutex::new(saver),
            problem: Mutex::new(problem),
            watcher: Mutex::new(None),
            jira: Mutex::new(jira),
            jira_path,
            settings: Mutex::new(settings),
            settings_path,
        })
    }

    /// A copy of the learned Jira projects.
    pub fn jira(&self) -> JiraProjects {
        lock(&self.jira).clone()
    }

    /// A copy of the settings.
    pub fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    /// Change the settings and keep them. Fails if the file could not be read at start (it is then
    /// not overwritten) or not written.
    pub fn set_settings(&self, settings: Settings) -> Result<(), String> {
        let path = self
            .settings_path
            .as_ref()
            .ok_or("the settings file is unreadable and stays untouched")?;
        settings.save(path).map_err(|e| e.to_string())?;
        *lock(&self.settings) = settings;
        Ok(())
    }

    /// Forget a learned Jira project and keep the change. Returns whether it was known.
    pub fn forget_jira(&self, project: &str) -> Result<bool, String> {
        let path = self
            .jira_path
            .as_ref()
            .ok_or("the Jira file is unreadable and stays untouched")?;
        let mut jira = lock(&self.jira);
        let known = jira.forget(project);
        if known {
            jira.save(path).map_err(|e| e.to_string())?;
        }
        Ok(known)
    }

    /// Learn the Jira projects of these links and keep them if something new was learned.
    pub fn learn_jira<'a>(&self, urls: impl IntoIterator<Item = &'a str>) {
        let mut jira = lock(&self.jira);
        let learned = urls.into_iter().filter(|u| jira.learn(u).is_some()).count();
        if learned > 0
            && let Some(path) = &self.jira_path
            && let Err(e) = jira.save(path)
        {
            eprintln!("tasqlet: Jira projects not saved: {e}");
        }
    }

    fn session(&self) -> MutexGuard<'_, Session> {
        lock(&self.session)
    }

    /// Work on the session directly (the command line) and save what it changed; the caller tells
    /// tray and webview with [`changed`].
    pub fn change<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        let mut session = self.session();
        let result = f(&mut session);
        self.save(session.store());
        result
    }

    /// A copy of the current store.
    pub fn store(&self) -> Store {
        self.session().store().clone()
    }

    /// The snapshot the webview renders: the open items in the data file's shape, each resource with
    /// the label to show (`shown`, own or smart label from the core), plus the load problem. Done
    /// items stay out – they grow with every year and the interface reaches them through the
    /// search – except `include`, the one the elaboration shows. (Done items carry no recipient:
    /// finishing ends the delegation, so the suggestions lose nothing.)
    pub fn snapshot(&self, include: Option<Id>) -> Value {
        let session = self.session();
        let store = session.store();
        let items: Vec<Value> = store
            .items()
            .iter()
            .filter(|it| it.state != State::Done || Some(it.id) == include)
            .map(|it| item_json(it))
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

/// An item as the webview gets it: the data file's shape, its resources most important first, each
/// with the label to show (`shown`) and its fixed `Alt+n` digit (docs/KONZEPT.md 5).
pub use tasqlet_core::cli::item_json;

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

/// Carry out a continuous edit (typing in the note): saved, but no undo step and no refresh.
pub fn edit(app: &AppHandle, action: Action) -> Result<(), Error> {
    let data = app.state::<Data>();
    let mut session = data.session();
    session.edit(action, Utc::now())?;
    data.save(session.store());
    Ok(())
}

/// Undo the last action.
pub fn undo(app: &AppHandle) -> Result<(), Error> {
    let data = app.state::<Data>();
    {
        let mut session = data.session();
        session.undo()?;
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
        let snapshot = data.snapshot(None);
        let resources = &snapshot["items"][0]["resources"];
        assert_eq!(resources[0]["shown"], "PR #42");
        assert_eq!(resources[1]["shown"], "Mail: Ops");
        assert_eq!(snapshot["load_error"], Value::Null);
    }

    #[test]
    fn done_items_stay_out_of_the_snapshot_unless_shown() {
        let dir = tempfile::tempdir().unwrap();
        let data = Data::open(dir.path().to_path_buf()).unwrap();
        let open = NewItem::new("offen");
        let done = NewItem::new("fertig");
        let done_id = done.id;
        let mut session = data.session();
        session
            .apply(
                Action::Add {
                    item: open,
                    target: Target::Focus,
                },
                Utc::now(),
            )
            .unwrap();
        session
            .apply(
                Action::Add {
                    item: done,
                    target: Target::Focus,
                },
                Utc::now(),
            )
            .unwrap();
        session.apply(Action::Finish(done_id), Utc::now()).unwrap();
        drop(session);
        let titles = |v: &Value| {
            v["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|it| it["title"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(&data.snapshot(None)), ["offen"]);
        assert_eq!(titles(&data.snapshot(Some(done_id))).len(), 2);
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
        assert_eq!(data.snapshot(None)["load_error"]["backup"], true);
        assert!(data.store().items().is_empty());
    }
}
