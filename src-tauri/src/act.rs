//! Changes to one item from the interface (work card, plan 1.7): a thin bridge from the webview's
//! requests to the core's [`Action`]s. Every rule lives in the core.

use serde::Deserialize;
use tasqlet_core::{Action, Error, Id, List};

use crate::data;

/// The lists an item can be sent to.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dest {
    Focus,
    Intermezzo,
    Idea,
}

/// What the interface asks for.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Act {
    Start { id: Id },
    Finish { id: Id },
    Pause,
    Resume,
    Delegate { id: Id, to: String },
    SendTo { id: Id, to: Dest },
    ToBack { id: Id },
    ToNext { id: Id },
    MoveBy { id: Id, delta: i32 },
    Rename { id: Id, title: String },
    SetNote { id: Id, note: String },
    AddCheck { id: Id, text: String },
    ToggleCheck { id: Id, check: Id },
    AddResource { id: Id, target: String },
}

impl Act {
    pub fn into_action(self) -> Action {
        let list = |d: Dest| match d {
            Dest::Focus => List::Focus,
            Dest::Intermezzo => List::Intermezzo,
            Dest::Idea => List::Idea,
        };
        match self {
            Act::Start { id } => Action::Start(id),
            Act::Finish { id } => Action::Finish(id),
            Act::Pause => Action::Pause,
            Act::Resume => Action::Resume,
            Act::Delegate { id, to } => Action::Delegate { id, to },
            Act::SendTo { id, to } => Action::SendTo { id, list: list(to) },
            Act::ToBack { id } => Action::ToBack(id),
            Act::ToNext { id } => Action::ToNext(id),
            Act::MoveBy { id, delta } => Action::MoveBy { id, delta },
            Act::Rename { id, title } => Action::Rename { id, title },
            Act::SetNote { id, note } => Action::SetNote { id, note },
            Act::AddCheck { id, text } => Action::AddCheck { id, text },
            Act::ToggleCheck { id, check } => Action::ToggleCheck { id, check },
            Act::AddResource { id, target } => Action::AddResource { id, target },
        }
    }

    /// Typing, not a step of its own: no undo step, no refresh of what the webview already shows.
    fn is_edit(&self) -> bool {
        matches!(self, Act::SetNote { .. })
    }
}

/// Carry out a change. `Unchanged` counts as success: the item is where the user wanted it.
#[tauri::command]
pub fn act(app: tauri::AppHandle, act: Act) -> Result<(), String> {
    let edit = act.is_edit();
    let result = if edit {
        data::edit(&app, act.into_action())
    } else {
        data::apply(&app, act.into_action())
    };
    match result {
        Ok(()) | Err(Error::Unchanged) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_webview_sends_tagged_json() {
        let id = Id::generate();
        let json = format!(r#"{{"kind":"send_to","id":"{id}","to":"idea"}}"#);
        let act: Act = serde_json::from_str(&json).unwrap();
        assert_eq!(
            act.into_action(),
            Action::SendTo {
                id,
                list: List::Idea
            }
        );
        assert!(matches!(
            serde_json::from_str::<Act>(r#"{"kind":"pause"}"#).unwrap(),
            Act::Pause
        ));
    }
}
