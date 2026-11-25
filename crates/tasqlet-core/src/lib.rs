//! Domain logic of Tasqlet, free of any GUI or desktop dependency.
//!
//! The specification lives in `docs/KONZEPT.md`; `docs/PLAN.md` says which part is built when.
//! So far: the item model with its transitions, invariants and undo (plan step 1.1) and storage on
//! disk in [`storage`] (1.2), the quick-entry parser in [`quick`] and smart labels and rank of the
//! resources in [`resources`] (1.3). Themes (2.6) follow.

pub mod jira;
mod model;
pub mod overview;
pub mod quick;
pub mod resources;
pub mod search;
mod session;
pub mod settings;
pub mod storage;
mod store;

pub use model::{
    ChecklistItem, Id, Item, NewItem, NewResource, Resource, ResourceType, State, Target,
    title_from_note,
};
pub use session::{Session, UNDO_LIMIT};
pub use store::{Action, Error, InvariantViolation, List, Store};

/// Version of the data file format (`schema` in `items.json`, see `docs/KONZEPT.md` 2.2).
pub const SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_starts_at_one() {
        assert_eq!(SCHEMA_VERSION, 1);
    }
}
