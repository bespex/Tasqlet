//! The store of a running app together with its undo history (`docs/KONZEPT.md` 2.4).
//!
//! The history keeps snapshots of the whole store. With a few hundred items that is cheap, and it
//! makes undo exact for every action without writing an inverse for each one. It lives in memory
//! only and is lost on restart.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use crate::store::{Action, Error, Store};

/// How many steps can be undone.
pub const UNDO_LIMIT: usize = 50;

/// A store with undo.
#[derive(Debug, Clone, Default)]
pub struct Session {
    store: Store,
    history: VecDeque<Store>,
}

impl Session {
    /// A session on the given store, with an empty history.
    pub fn new(store: Store) -> Self {
        Self {
            store,
            history: VecDeque::new(),
        }
    }

    /// The current state.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Carry out an action and remember the state before it. A refused action leaves no trace.
    pub fn apply(&mut self, action: Action, now: DateTime<Utc>) -> Result<(), Error> {
        let before = self.store.clone();
        self.store.apply(action, now)?;
        if self.history.len() == UNDO_LIMIT {
            self.history.pop_front();
        }
        self.history.push_back(before);
        Ok(())
    }

    /// Carry out a continuous edit such as the note's autosave: not a step of its own in the
    /// history. The edit is also made in the remembered states, so undoing an earlier action
    /// does not take the typed text away.
    pub fn edit(&mut self, action: Action, now: DateTime<Utc>) -> Result<(), Error> {
        self.store.apply(action.clone(), now)?;
        for before in &mut self.history {
            let _ = before.apply(action.clone(), now);
        }
        Ok(())
    }

    /// Go back to the state before the last action.
    pub fn undo(&mut self) -> Result<(), Error> {
        self.store = self.history.pop_back().ok_or(Error::NothingToUndo)?;
        Ok(())
    }

    /// Whether there is something to undo.
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Replace the state, e.g. after the data file was changed from outside. The history is
    /// dropped, since its snapshots no longer fit.
    pub fn replace(&mut self, store: Store) {
        self.store = store;
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewItem, Target};

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_795_000_000, 0).unwrap()
    }

    #[test]
    fn undo_history_shares_the_items_an_action_did_not_touch() {
        let mut s = Session::default();
        let (a, b) = (NewItem::new("a"), NewItem::new("b"));
        let (ida, idb) = (a.id, b.id);
        for item in [a, b] {
            s.apply(
                Action::Add {
                    item,
                    target: Target::Idea,
                },
                now(),
            )
            .unwrap();
        }
        s.apply(
            Action::Rename {
                id: ida,
                title: "A".into(),
            },
            now(),
        )
        .unwrap();
        let before = s.history.back().unwrap();
        let find = |store: &Store, id| store.items().iter().find(|it| it.id == id).unwrap().clone();
        assert!(
            std::sync::Arc::ptr_eq(&find(s.store(), idb), &find(before, idb)),
            "untouched: shared"
        );
        assert!(
            !std::sync::Arc::ptr_eq(&find(s.store(), ida), &find(before, ida)),
            "renamed: copied"
        );
    }

    #[test]
    fn undo_restores_the_state_before_each_action() {
        let mut s = Session::default();
        let a = NewItem::new("a");
        let b = NewItem::new("b");
        let (ida, idb) = (a.id, b.id);
        s.apply(
            Action::Add {
                item: a,
                target: Target::Now,
            },
            now(),
        )
        .unwrap();
        s.apply(
            Action::Add {
                item: b,
                target: Target::Focus,
            },
            now(),
        )
        .unwrap();
        let before = s.store().clone();
        s.apply(Action::Start(idb), now()).unwrap();
        assert_eq!(s.store().active().map(|it| it.id), Some(idb));
        s.undo().unwrap();
        assert_eq!(s.store(), &before);
        assert_eq!(s.store().active().map(|it| it.id), Some(ida));
        s.undo().unwrap();
        s.undo().unwrap();
        assert!(s.store().items().is_empty());
        assert_eq!(s.undo(), Err(Error::NothingToUndo));
    }

    #[test]
    fn edits_leave_no_step_and_survive_undo() {
        let mut s = Session::default();
        let a = NewItem::new("a");
        let id = a.id;
        s.apply(
            Action::Add {
                item: a,
                target: Target::Now,
            },
            now(),
        )
        .unwrap();
        s.apply(
            Action::Rename {
                id,
                title: "b".into(),
            },
            now(),
        )
        .unwrap();
        s.edit(
            Action::SetNote {
                id,
                note: "typed".into(),
            },
            now(),
        )
        .unwrap();
        s.undo().unwrap();
        let item = s.store().get(id).unwrap();
        assert_eq!((item.title.as_str(), item.note.as_str()), ("a", "typed"));
        s.undo().unwrap();
        assert!(s.store().items().is_empty());
        assert!(!s.can_undo());
    }

    #[test]
    fn refused_actions_are_not_recorded() {
        let mut s = Session::default();
        assert_eq!(s.apply(Action::Pause, now()), Err(Error::NoActiveItem));
        assert!(!s.can_undo());
    }

    #[test]
    fn history_keeps_the_last_fifty_steps() {
        let mut s = Session::default();
        for i in 0..UNDO_LIMIT + 10 {
            let item = NewItem::new(format!("{i}"));
            s.apply(
                Action::Add {
                    item,
                    target: Target::Idea,
                },
                now(),
            )
            .unwrap();
        }
        let mut undone = 0;
        while s.undo().is_ok() {
            undone += 1;
        }
        assert_eq!(undone, UNDO_LIMIT);
        assert_eq!(s.store().items().len(), 10);
    }

    #[test]
    fn replace_drops_the_history() {
        let mut s = Session::default();
        s.apply(
            Action::Add {
                item: NewItem::new("a"),
                target: Target::Idea,
            },
            now(),
        )
        .unwrap();
        s.replace(Store::new());
        assert!(!s.can_undo());
    }
}
