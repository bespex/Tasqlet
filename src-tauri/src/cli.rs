//! The command line (plan 2.5): `tasqlet add|open|current|list`. Reading the arguments and the
//! commands themselves are the core's (`tasqlet_core::cli`); here they are carried out in the
//! running app, which got them over its socket, or, if none runs, on the files directly. The
//! single-instance lock is held meanwhile, so no app starts in between.

use std::path::Path;

use chrono::{Local, Utc};
use tasqlet_core::cli::{self, Command, Outcome};
use tasqlet_core::jira::{self, JIRA_FILE, JiraProjects};
use tasqlet_core::storage::Storage;
use tasqlet_core::{Action, Session};
use tauri::{AppHandle, Manager};

use crate::data::{self, Data};
use crate::open;

/// In the running app: through its session (undo, saving, tray and popover follow).
pub fn run_in_app(app: &AppHandle, command: &Command) -> Result<String, String> {
    let data = app.state::<Data>();
    let jira = data.jira();
    let outcome = data.change(|session| {
        cli::run(
            session,
            &jira,
            command,
            Local::now().date_naive(),
            Utc::now(),
        )
    })?;
    data.learn_jira(outcome.learn.iter().map(String::as_str));
    if outcome.changed {
        data::changed(app);
    }
    let mut failed = None;
    for (item, resource, _) in &outcome.open {
        if let Err(e) = open::open_one(app, *item, *resource) {
            failed.get_or_insert(e);
        }
    }
    failed.map_or(Ok(outcome.out), Err)
}

/// Without a running app: load, carry out, save. A file Tasqlet cannot read is left alone.
pub fn run_direct(data_dir: &Path, command: &Command) -> Result<String, String> {
    let storage = Storage::open(data_dir).map_err(|e| e.to_string())?;
    let loaded = storage
        .load()
        .map_err(|e| format!("{e}; nothing changed"))?;
    let mut session = Session::new(loaded.store);
    let jira_path = jira::config_dir().map(|dir| dir.join(JIRA_FILE));
    let mut jira = jira_path
        .as_deref()
        .and_then(|p| JiraProjects::load(p).ok())
        .unwrap_or_default();
    let Outcome {
        out,
        open,
        learn,
        mut changed,
    } = cli::run(
        &mut session,
        &jira,
        command,
        Local::now().date_naive(),
        Utc::now(),
    )?;
    if learn.iter().filter(|u| jira.learn(u).is_some()).count() > 0
        && let Some(path) = &jira_path
    {
        let _ = jira.save(path);
    }
    let mut failed = None;
    for (item, resource, target) in &open {
        let kind = session
            .store()
            .get(*item)
            .and_then(|it| it.resources.iter().find(|r| r.id == *resource))
            .map(|r| r.kind);
        let result = kind
            .ok_or_else(|| "unknown resource".to_string())
            .and_then(|kind| open::launch_checked(kind, target));
        match result {
            Ok(()) => {
                changed |= session
                    .edit(
                        Action::MarkUsed {
                            id: *item,
                            resource: *resource,
                        },
                        Utc::now(),
                    )
                    .is_ok();
            }
            Err(e) => {
                failed.get_or_insert(e);
            }
        }
    }
    if changed {
        storage.save(session.store()).map_err(|e| e.to_string())?;
    }
    failed.map_or(Ok(out), Err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tasqlet_core::State;
    use tasqlet_core::cli::Place;

    #[test]
    fn without_the_app_the_files_are_changed_directly() {
        let dir = tempfile::tempdir().unwrap();
        let add = Command::Add {
            text: "From a script".into(),
            place: Place::Focus,
            links: vec!["https://ci.example.com/42".into()],
        };
        assert_eq!(
            run_direct(dir.path(), &add).unwrap(),
            "Added to focus #1: From a script\n"
        );
        let store = Storage::open(dir.path()).unwrap().load().unwrap().store;
        let focus = store.list(State::Focus);
        assert_eq!(focus[0].title, "From a script");
        assert_eq!(focus[0].resources[0].target, "https://ci.example.com/42");
        let out = run_direct(dir.path(), &Command::Current { json: false }).unwrap();
        assert!(out.starts_with("From a script\n"));
    }

    #[test]
    fn an_unreadable_file_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::open(dir.path()).unwrap();
        std::fs::write(storage.items_path(), "{ broken").unwrap();
        let add = Command::Add {
            text: "x".into(),
            place: Place::Focus,
            links: vec![],
        };
        assert!(run_direct(dir.path(), &add).is_err());
        assert_eq!(
            std::fs::read_to_string(storage.items_path()).unwrap(),
            "{ broken"
        );
    }
}
