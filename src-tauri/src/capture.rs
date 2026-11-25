//! Quick entry (docs/KONZEPT.md 3.1, plan 1.6): recognise shortcuts while typing, save with one of
//! the four keys, undo. Parsing and placing happen in the core; this is only the bridge.

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tasqlet_core::quick::{QuickEntry, parse_with};
use tasqlet_core::resources::smart_label;
use tasqlet_core::{Action, Error, ResourceType, State, Target};
use tauri::{AppHandle, Manager};

use crate::data::{self, Data};

/// The key used to save (docs/KONZEPT.md 3.1), from urgent to later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Key {
    /// `Ctrl Shift Enter`: active on #1 right away.
    Now,
    /// `Ctrl Enter`: end of the focus list.
    Focus,
    /// `Alt Enter`: intermezzo bar.
    Intermezzo,
    /// `Enter`: ideas.
    Idea,
}

/// What was saved, for the feedback line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Captured {
    pub title: String,
    /// Where it went: `now`, `focus`, `intermezzo`, `idea` or `delegated`.
    pub placed: &'static str,
    /// Position in the focus list (1-based) for `now` and `focus`.
    pub position: Option<usize>,
    /// Whether another task was active before (`now`: it moved to #2).
    pub replaced: bool,
    pub delegated_to: Option<String>,
}

fn today() -> chrono::NaiveDate {
    Local::now().date_naive()
}

/// The entry as the webview shows it while typing: each resource with the label to show.
fn entry_json(entry: &QuickEntry) -> Value {
    let mut value = serde_json::to_value(entry).unwrap_or(Value::Null);
    if let Some(resources) = value["resources"].as_array_mut() {
        for (json, resource) in resources.iter_mut().zip(&entry.resources) {
            json["shown"] = smart_label(&resource.target).into();
        }
    }
    value
}

/// What the quick entry recognises in `text`, live while typing.
#[tauri::command]
pub fn parse_quick(app: AppHandle, text: String) -> Value {
    let jira = app.state::<Data>().jira();
    entry_json(&parse_with(&text, today(), &jira))
}

/// Save the quick entry with the given key.
#[tauri::command]
pub fn capture(app: AppHandle, text: String, key: Key) -> Result<Captured, String> {
    let data = app.state::<Data>();
    let entry = parse_with(&text, today(), &data.jira());
    if entry.is_empty() {
        return Err("nothing to save".into());
    }
    data.learn_jira(
        entry
            .resources
            .iter()
            .filter(|r| r.kind == ResourceType::Url)
            .map(|r| r.target.as_str()),
    );
    let chosen = match key {
        Key::Now => Target::Now,
        Key::Focus => Target::Focus,
        Key::Intermezzo => Target::Intermezzo,
        Key::Idea => Target::Idea,
    };
    let target = entry.target(chosen);
    let replaced = target == Target::Now && data.store().active().is_some();
    let title = entry.title.clone();
    let delegated_to = entry.delegate_to.clone();
    let item = entry.into_new_item();
    let id = item.id;
    data::apply(
        &app,
        Action::Add {
            item,
            target: target.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    let store = data.store();
    let position = store
        .list(State::Focus)
        .iter()
        .position(|it| it.id == id)
        .map(|i| i + 1);
    let placed = match target {
        Target::Now => "now",
        Target::Focus => "focus",
        Target::Intermezzo => "intermezzo",
        Target::Idea => "idea",
        Target::Delegated(_) => "delegated",
    };
    Ok(Captured {
        title,
        placed,
        position,
        replaced,
        delegated_to,
    })
}

/// Undo the last action (`Ctrl+Z`, "Undo" in the feedback line).
#[tauri::command]
pub fn undo(app: AppHandle) -> Result<(), String> {
    data::undo(&app).map_err(|e| match e {
        Error::NothingToUndo => "nothing to undo".to_string(),
        e => e.to_string(),
    })
}
