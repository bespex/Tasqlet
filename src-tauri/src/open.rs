//! Opening resources (docs/KONZEPT.md 5, plan 1.9): `xdg-open` with the link, file or mail, and
//! `used_at` set so the resource ranks first next time. Domain rules stay in the core; this only
//! starts the program.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use tasqlet_core::{Action, Id, ResourceType, resources::ranked};
use tauri::{AppHandle, Manager};

use crate::data::{self, Data};

/// What `xdg-open` gets: `~/` expanded, everything else as written. Refuses text that could pass
/// for an option.
fn target_for_open(target: &str) -> Result<String, String> {
    let target = target.trim();
    if target.is_empty() || target.starts_with('-') {
        return Err("not a resource".into());
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    Ok(match (target.strip_prefix("~/"), home) {
        (Some(rest), Some(home)) => home.join(rest).to_string_lossy().into_owned(),
        _ => target.to_string(),
    })
}

/// Start `xdg-open` detached; a thread reaps it so no zombie stays behind.
fn launch(target: &str) -> Result<(), String> {
    let target = target_for_open(target)?;
    let mut child = Command::new("xdg-open")
        .arg(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("xdg-open: {e}"))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

pub fn open_one(app: &AppHandle, id: Id, resource: Id) -> Result<(), String> {
    let target = {
        let store = app.state::<Data>().store();
        let item = store.get(id).ok_or("unknown item")?;
        let r = item
            .resources
            .iter()
            .find(|r| r.id == resource)
            .ok_or("unknown resource")?;
        // A file that is gone is told to the user, not handed to the desktop.
        if r.kind == ResourceType::File {
            let path = target_for_open(r.target.strip_prefix("file://").unwrap_or(&r.target))?;
            if !std::path::Path::new(&path).exists() {
                return Err(format!("not found: {}", r.target));
            }
        }
        r.target.clone()
    };
    launch(&target)?;
    // Opening is no undo step of its own.
    let _ = data::edit(app, Action::MarkUsed { id, resource });
    Ok(())
}

/// Open one resource of an item.
#[tauri::command]
pub fn open_resource(app: AppHandle, id: Id, resource: Id) -> Result<(), String> {
    open_one(&app, id, resource)
}

/// Open all resources of an item, most important first.
#[tauri::command]
pub fn open_all(app: AppHandle, id: Id) -> Result<(), String> {
    let ids: Vec<Id> = {
        let store = app.state::<Data>().store();
        let item = store.get(id).ok_or("unknown item")?;
        ranked(&item.resources)
            .into_iter()
            .map(|r| r.resource.id)
            .collect()
    };
    let mut failed = None;
    for resource in ids {
        if let Err(e) = open_one(&app, id, resource) {
            failed.get_or_insert(e);
        }
    }
    failed.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_the_home_folder_and_refuses_options() {
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(
            target_for_open("~/docs/a.md").unwrap(),
            format!("{home}/docs/a.md")
        );
        assert_eq!(
            target_for_open("https://example.com").unwrap(),
            "https://example.com"
        );
        assert!(target_for_open("--help").is_err());
        assert!(target_for_open("  ").is_err());
    }
}
