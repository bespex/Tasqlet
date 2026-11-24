//! Item model as described in `docs/KONZEPT.md` 2.1 and 2.2.
//!
//! Field names follow the data file (`items.json`), so the types serialize to exactly that shape.
//! The invariants between items (one `active`, gap-free `order`, …) are kept by [`crate::Store`],
//! which is why items are only handed out by reference.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Identifier of an item, a checklist entry or a resource.
pub type Id = Ulid;

/// Where an item lives (`docs/KONZEPT.md` 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Raw thought, a growing note. Tab "Ideen", newest first.
    Idea,
    /// Urgent, in a fixed order (#1, #2, …). Tab "Fokus", main list.
    Focus,
    /// Small, no hurry. Tab "Fokus", bar at the bottom.
    Intermezzo,
    /// Handed to someone, kept in view. Tab "Delegiert".
    Delegated,
    /// Finished, stays searchable.
    Done,
}

impl State {
    /// Whether items in this state form a list with a gap-free `order` (focus and intermezzo).
    pub fn is_ordered(self) -> bool {
        matches!(self, State::Focus | State::Intermezzo)
    }
}

/// Kind of a work resource attached to an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    /// A web link.
    Url,
    /// A file or a folder.
    File,
    /// A mail: `.eml` file, `mailto:` or `mid:` link.
    Mail,
}

/// A link, file or mail that belongs to an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub id: Id,
    #[serde(rename = "type")]
    pub kind: ResourceType,
    pub target: String,
    /// Own label; `None` means the smart label derived from `target` is shown.
    pub label: Option<String>,
    /// Last time it was opened from Tasqlet; ranks the visible resources.
    pub used_at: Option<DateTime<Utc>>,
}

/// One line of an item's checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Id,
    pub text: String,
    #[serde(default)]
    pub done: bool,
}

/// A task, idea or note.
///
/// Fields that may be left out of a hand-written file have serde defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: Id,
    /// May be empty; promoting an item without title takes the first note line.
    #[serde(default)]
    pub title: String,
    pub state: State,
    /// Only in focus, at most one, always at `order` 0.
    #[serde(default)]
    pub active: bool,
    /// Only in focus, at most one: the task left when another one was taken over.
    #[serde(default)]
    pub last_active: bool,
    /// Gap-free from 0 within focus and within intermezzo; 0 in the other states.
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default)]
    pub resources: Vec<Resource>,
    /// Only noted, never evaluated (D2).
    pub date: Option<NaiveDate>,
    pub delegated_to: Option<String>,
    pub delegated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub done_at: Option<DateTime<Utc>>,
}

impl Item {
    /// The title, or the first note line if the title is blank (see [`title_from_note`]).
    pub fn effective_title(&self) -> String {
        if self.title.trim().is_empty() {
            title_from_note(&self.note)
        } else {
            self.title.clone()
        }
    }
}

/// Title derived from a note: its first non-blank line, without a leading list marker.
///
/// Returns an empty string for an empty note; the interface shows its own placeholder then.
pub fn title_from_note(note: &str) -> String {
    note.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| {
            line.trim_start_matches(['-', '*', '•'])
                .trim_start()
                .to_string()
        })
        .unwrap_or_default()
}

/// Where a newly captured item goes (`docs/KONZEPT.md` 3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Active on #1 right away; the previous #1 moves to #2.
    Now,
    /// End of the focus list; the current work stays.
    Focus,
    /// End of the intermezzo bar.
    Intermezzo,
    /// Into the ideas.
    Idea,
    /// Delegated to the given person.
    Delegated(String),
}

/// Content of an item to be added. The id is chosen up front, so adding is deterministic and the
/// caller knows the new item without a lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewItem {
    pub id: Id,
    pub title: String,
    pub note: String,
    pub date: Option<NaiveDate>,
    pub checklist: Vec<String>,
    pub resources: Vec<NewResource>,
}

impl NewItem {
    /// A new item with a fresh id and the given title, everything else empty.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Ulid::generate(),
            title: title.into(),
            note: String::new(),
            date: None,
            checklist: Vec::new(),
            resources: Vec::new(),
        }
    }
}

/// A resource to attach to a new item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewResource {
    pub kind: ResourceType,
    pub target: String,
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_from_note_takes_first_non_blank_line_without_marker() {
        assert_eq!(
            title_from_note("\n  - Farben sammeln\n- zweite"),
            "Farben sammeln"
        );
        assert_eq!(title_from_note("• Punkt"), "Punkt");
        assert_eq!(title_from_note("Satz ohne Marker"), "Satz ohne Marker");
        assert_eq!(title_from_note("   \n"), "");
    }

    #[test]
    fn state_and_resource_type_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&State::Intermezzo).unwrap(),
            "\"intermezzo\""
        );
        assert_eq!(
            serde_json::to_string(&ResourceType::Mail).unwrap(),
            "\"mail\""
        );
    }
}
