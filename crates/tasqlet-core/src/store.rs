//! All items plus the global break flag, with the transitions of `docs/KONZEPT.md` 2.4 and the
//! invariants of 2.3.
//!
//! Every transition first checks whether it applies and only then changes anything, so an `Err`
//! always leaves the store as it was. After each change the ordered lists are renumbered.

use std::collections::HashSet;
use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};

use crate::model::{ChecklistItem, Id, Item, NewItem, Resource, State, Target, title_from_note};

/// A change to the store, as the interface asks for it. Every action is undoable
/// (see [`crate::Session`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Capture a new item.
    Add { item: NewItem, target: Target },
    /// "Übernehmen": make the item active on #1.
    Start(Id),
    /// "Als Nächstes": put the item on #2, the current work stays.
    ToNext(Id),
    /// "Verschieben": move a focus item within #2 … n by `delta` places.
    MoveBy { id: Id, delta: i32 },
    /// "Erledigt".
    Finish(Id),
    /// "Delegieren" to a person (`@Name` or `Name`).
    Delegate { id: Id, to: String },
    /// "Zurück in Fokus", "→ Fokus", "⚡ Zw.", "→ Idee": to the end of the given list.
    SendTo { id: Id, list: List },
    /// Start the coffee break.
    Pause,
    /// End the break and go on with the same task.
    Resume,
    /// "Zurück in die Queue": a focus item goes to the end of the focus list, no longer active.
    ToBack(Id),
    /// Change the title.
    Rename { id: Id, title: String },
    /// Replace the note (autosave, see [`Session::edit`](crate::Session::edit)).
    SetNote { id: Id, note: String },
    /// Add a step to the checklist.
    AddCheck { id: Id, text: String },
    /// Tick or untick a step.
    ToggleCheck { id: Id, check: Id },
    /// Change the wording of a step.
    RenameCheck { id: Id, check: Id, text: String },
    /// Take a step off the checklist.
    RemoveCheck { id: Id, check: Id },
    /// Note a date, or clear it. Only noted, never evaluated (D2).
    SetDate { id: Id, date: Option<NaiveDate> },
    /// Attach a link, file or mail; the kind is derived from the text.
    AddResource { id: Id, target: String },
    /// A brainstorm thought: a link, file or mail becomes a resource, anything else a bullet in the note.
    Brainstorm { id: Id, text: String },
    /// Take a resource off an item.
    RemoveResource { id: Id, resource: Id },
    /// A resource was opened: it ranks first from now on (`used_at`).
    MarkUsed { id: Id, resource: Id },
}

/// Destination of [`Action::SendTo`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum List {
    Focus,
    Intermezzo,
    Idea,
}

impl From<List> for State {
    fn from(list: List) -> Self {
        match list {
            List::Focus => State::Focus,
            List::Intermezzo => State::Intermezzo,
            List::Idea => State::Idea,
        }
    }
}

/// Why an action was not carried out. The store is unchanged in every case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// No item with this id.
    UnknownItem(Id),
    /// An item with this id exists already.
    DuplicateId(Id),
    /// The item is already where the action would put it.
    Unchanged,
    /// The action does not apply to the item in its current place (e.g. moving #1).
    NotApplicable,
    /// A break needs an active task.
    NoActiveItem,
    /// Delegating needs a name.
    EmptyName,
    /// The undo history is empty.
    NothingToUndo,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownItem(id) => write!(f, "no item with id {id}"),
            Error::DuplicateId(id) => write!(f, "an item with id {id} exists already"),
            Error::Unchanged => f.write_str("nothing to change"),
            Error::NotApplicable => f.write_str("action does not apply to this item"),
            Error::NoActiveItem => f.write_str("there is no active task"),
            Error::EmptyName => f.write_str("a name is needed to delegate"),
            Error::NothingToUndo => f.write_str("nothing to undo"),
        }
    }
}

impl std::error::Error for Error {}

/// A broken invariant of `docs/KONZEPT.md` 2.3, found by [`Store::check_invariants`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantViolation(pub String);

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invariant violated: {}", self.0)
    }
}

impl std::error::Error for InvariantViolation {}

/// All items and the break flag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Store {
    on_break: bool,
    items: Vec<Item>,
}

impl Store {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// A store from loaded data, refused if it breaks an invariant.
    pub fn from_parts(on_break: bool, items: Vec<Item>) -> Result<Self, InvariantViolation> {
        let store = Self { on_break, items };
        store.check_invariants()?;
        Ok(store)
    }

    /// A store from data that may have been edited by hand: whatever breaks an invariant is
    /// repaired instead of refused. Duplicate ids keep their first item, the active item with the
    /// lowest `order` stays active, flags outside focus are cleared and the lists renumbered.
    /// Returns the store and whether anything had to be repaired.
    pub fn repaired(on_break: bool, items: Vec<Item>) -> (Self, bool) {
        let original = items.clone();
        let mut seen = HashSet::new();
        let mut items: Vec<Item> = items.into_iter().filter(|it| seen.insert(it.id)).collect();
        for it in &mut items {
            if it.state != State::Focus {
                it.active = false;
                it.last_active = false;
            }
            if !it.state.is_ordered() {
                it.order = 0;
            }
        }
        let active = items
            .iter()
            .filter(|it| it.active)
            .min_by_key(|it| it.order)
            .map(|it| it.id);
        let last_active = items
            .iter()
            .find(|it| it.last_active && Some(it.id) != active)
            .map(|it| it.id);
        for it in &mut items {
            it.active = Some(it.id) == active;
            it.last_active = Some(it.id) == last_active;
        }
        let mut store = Self {
            on_break: on_break && active.is_some(),
            items,
        };
        let mut focus: Vec<&Item> = store.list(State::Focus);
        focus.sort_by_key(|it| (!it.active, it.order));
        let focus: Vec<Id> = focus.into_iter().map(|it| it.id).collect();
        store.set_order(&focus);
        store.renumber();
        let changed = store.on_break != on_break || store.items != original;
        (store, changed)
    }

    /// Whether the coffee break is on.
    pub fn on_break(&self) -> bool {
        self.on_break
    }

    /// All items, in no particular order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The item with this id.
    pub fn get(&self, id: Id) -> Option<&Item> {
        self.items.iter().find(|it| it.id == id)
    }

    /// Items of one state; focus and intermezzo sorted by `order`, ideas newest first, delegated
    /// and done in no particular order.
    pub fn list(&self, state: State) -> Vec<&Item> {
        let mut list: Vec<&Item> = self.items.iter().filter(|it| it.state == state).collect();
        if state.is_ordered() {
            list.sort_by_key(|it| it.order);
        } else if state == State::Idea {
            list.sort_by_key(|it| std::cmp::Reverse(it.created_at));
        }
        list
    }

    /// The active task, if any.
    pub fn active(&self) -> Option<&Item> {
        self.items.iter().find(|it| it.active)
    }

    /// Carry out an action. On `Err` nothing has changed.
    pub fn apply(&mut self, action: Action, now: DateTime<Utc>) -> Result<(), Error> {
        match action {
            Action::Add { item, target } => self.add(item, target, now),
            Action::Start(id) => self.start(id, now),
            Action::ToNext(id) => self.to_next(id, now),
            Action::MoveBy { id, delta } => self.move_by(id, delta),
            Action::Finish(id) => self.finish(id, now),
            Action::Delegate { id, to } => self.delegate(id, &to, now),
            Action::SendTo { id, list } => self.send_to(id, list, now),
            Action::Pause => self.pause(),
            Action::Resume => self.resume(),
            Action::ToBack(id) => self.to_back(id, now),
            Action::Rename { id, title } => self.rename(id, &title, now),
            Action::SetNote { id, note } => self.set_note(id, note, now),
            Action::AddCheck { id, text } => self.add_check(id, &text, now),
            Action::ToggleCheck { id, check } => self.toggle_check(id, check, now),
            Action::RenameCheck { id, check, text } => self.rename_check(id, check, &text, now),
            Action::RemoveCheck { id, check } => self.remove_check(id, check, now),
            Action::SetDate { id, date } => self.set_date(id, date, now),
            Action::AddResource { id, target } => self.add_resource(id, &target, now),
            Action::Brainstorm { id, text } => self.brainstorm(id, &text, now),
            Action::RemoveResource { id, resource } => self.remove_resource(id, resource, now),
            Action::MarkUsed { id, resource } => self.mark_used(id, resource, now),
        }
    }

    /// "Zurück in die Queue": to the end of the focus list. The next one stands on #1 as "Bereit".
    pub fn to_back(&mut self, id: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        if self.items[i].state != State::Focus {
            return Err(Error::NotApplicable);
        }
        let mut ids = self.ordered_ids(State::Focus);
        if ids.last() == Some(&id) {
            return Err(Error::Unchanged);
        }
        ids.retain(|&x| x != id);
        ids.push(id);
        if self.items[i].active {
            self.on_break = false;
        }
        let it = &mut self.items[i];
        it.active = false;
        it.last_active = false;
        it.updated_at = now;
        self.set_order(&ids);
        Ok(())
    }

    /// Change the title. An item outside the ideas needs one, so a blank title is refused there.
    pub fn rename(&mut self, id: Id, title: &str, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let title = title.trim();
        if title.is_empty() && self.items[i].state != State::Idea {
            return Err(Error::EmptyName);
        }
        if self.items[i].title == title {
            return Err(Error::Unchanged);
        }
        self.items[i].title = title.to_string();
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Replace the note.
    pub fn set_note(&mut self, id: Id, note: String, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        if self.items[i].note == note {
            return Err(Error::Unchanged);
        }
        self.items[i].note = note;
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Add a step at the end of the checklist; blank text is refused.
    pub fn add_check(&mut self, id: Id, text: &str, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::EmptyName);
        }
        self.items[i].checklist.push(ChecklistItem {
            id: Id::generate(),
            text: text.to_string(),
            done: false,
        });
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Tick or untick a step.
    pub fn toggle_check(&mut self, id: Id, check: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let step = self.items[i]
            .checklist
            .iter_mut()
            .find(|c| c.id == check)
            .ok_or(Error::UnknownItem(check))?;
        step.done = !step.done;
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Change the wording of a step. Blank text is refused, the step stays.
    pub fn rename_check(
        &mut self,
        id: Id,
        check: Id,
        text: &str,
        now: DateTime<Utc>,
    ) -> Result<(), Error> {
        let i = self.index(id)?;
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::EmptyName);
        }
        let step = self.items[i]
            .checklist
            .iter_mut()
            .find(|c| c.id == check)
            .ok_or(Error::UnknownItem(check))?;
        if step.text == text {
            return Err(Error::Unchanged);
        }
        step.text = text.to_string();
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Take a step off the checklist.
    pub fn remove_check(&mut self, id: Id, check: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let before = self.items[i].checklist.len();
        self.items[i].checklist.retain(|c| c.id != check);
        if self.items[i].checklist.len() == before {
            return Err(Error::UnknownItem(check));
        }
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Note a date or clear it. The date is only shown, never evaluated (D2).
    pub fn set_date(
        &mut self,
        id: Id,
        date: Option<NaiveDate>,
        now: DateTime<Utc>,
    ) -> Result<(), Error> {
        let i = self.index(id)?;
        if self.items[i].date == date {
            return Err(Error::Unchanged);
        }
        self.items[i].date = date;
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Add a thought to an item (`docs/KONZEPT.md` 6.2): a link, file or mail becomes a resource,
    /// anything else a new bullet at the end of the note, so nothing gets lost while an idea grows.
    pub fn brainstorm(&mut self, id: Id, text: &str, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::EmptyName);
        }
        if crate::resources::classify(text).is_some() {
            return self.add_resource(id, text, now);
        }
        let note = &mut self.items[i].note;
        if !note.is_empty() && !note.ends_with('\n') {
            note.push('\n');
        }
        note.push_str("- ");
        note.push_str(text);
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Take a resource off an item.
    pub fn remove_resource(
        &mut self,
        id: Id,
        resource: Id,
        now: DateTime<Utc>,
    ) -> Result<(), Error> {
        let i = self.index(id)?;
        let before = self.items[i].resources.len();
        self.items[i].resources.retain(|r| r.id != resource);
        if self.items[i].resources.len() == before {
            return Err(Error::UnknownItem(resource));
        }
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Note that a resource was opened now, so it ranks first (`docs/KONZEPT.md` 5).
    pub fn mark_used(&mut self, id: Id, resource: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let r = self.items[i]
            .resources
            .iter_mut()
            .find(|r| r.id == resource)
            .ok_or(Error::UnknownItem(resource))?;
        r.used_at = Some(now);
        Ok(())
    }

    /// Attach a resource; text that is no link, path or mail is refused. The same target twice
    /// is `Unchanged`.
    pub fn add_resource(&mut self, id: Id, target: &str, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let target = target.trim();
        let kind = crate::resources::classify(target).ok_or(Error::NotApplicable)?;
        if self.items[i].resources.iter().any(|r| r.target == target) {
            return Err(Error::Unchanged);
        }
        self.items[i].resources.push(Resource {
            id: Id::generate(),
            kind,
            target: target.to_string(),
            label: None,
            used_at: None,
        });
        self.items[i].updated_at = now;
        Ok(())
    }

    /// Capture a new item at the given target (`docs/KONZEPT.md` 3.1).
    pub fn add(&mut self, new: NewItem, target: Target, now: DateTime<Utc>) -> Result<(), Error> {
        if self.get(new.id).is_some() {
            return Err(Error::DuplicateId(new.id));
        }
        let delegate_to = match &target {
            Target::Delegated(name) => Some(person_name(name)?),
            _ => None,
        };
        let id = new.id;
        let state = match target {
            Target::Now | Target::Focus => State::Focus,
            Target::Intermezzo => State::Intermezzo,
            Target::Idea => State::Idea,
            Target::Delegated(_) => State::Delegated,
        };
        let mut title = new.title;
        if title.trim().is_empty() && state != State::Idea {
            title = title_from_note(&new.note);
        }
        self.items.push(Item {
            id,
            title,
            state,
            active: false,
            last_active: false,
            order: if state.is_ordered() { u32::MAX } else { 0 },
            note: new.note,
            checklist: new
                .checklist
                .into_iter()
                .map(|text| ChecklistItem {
                    id: Id::generate(),
                    text,
                    done: false,
                })
                .collect(),
            resources: new
                .resources
                .into_iter()
                .map(|r| Resource {
                    id: Id::generate(),
                    kind: r.kind,
                    target: r.target,
                    label: r.label,
                    used_at: None,
                })
                .collect(),
            date: new.date,
            delegated_at: delegate_to.as_ref().map(|_| now),
            delegated_to: delegate_to,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            done_at: None,
        });
        self.renumber();
        if target == Target::Now {
            self.start(id, now)?;
        }
        Ok(())
    }

    /// "Übernehmen": the item becomes active on #1, the previous #1 moves to #2. If that one was
    /// active, it is marked `last_active`. Ideas and intermezzo items become focus items. Starting
    /// any task ends the break; starting the active task during a break resumes it.
    pub fn start(&mut self, id: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        if self.items[i].active {
            return if self.on_break {
                self.on_break = false;
                Ok(())
            } else {
                Err(Error::Unchanged)
            };
        }
        self.on_break = false;
        let queue: Vec<Id> = self
            .ordered_ids(State::Focus)
            .into_iter()
            .filter(|&x| x != id)
            .collect();
        let left = queue.first().copied().filter(|&top| self.item(top).active);
        if let Some(left) = left {
            for it in &mut self.items {
                let marked = it.id == left;
                if it.last_active != marked || it.active {
                    it.last_active = marked;
                    it.active = false;
                    it.updated_at = now;
                }
            }
        }
        self.change_state(i, State::Focus, now);
        let it = &mut self.items[i];
        it.active = true;
        it.last_active = false;
        it.updated_at = now;
        let order: Vec<Id> = std::iter::once(id).chain(queue).collect();
        self.set_order(&order);
        self.renumber();
        Ok(())
    }

    /// "Als Nächstes": the item goes to #2 (to #1 as "Bereit" if the focus list is empty); the
    /// current work stays. Items from other states become focus items.
    pub fn to_next(&mut self, id: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let it = &self.items[i];
        if it.state == State::Focus {
            match it.order {
                0 => return Err(Error::NotApplicable),
                1 => return Err(Error::Unchanged),
                _ => {}
            }
        }
        let mut queue: Vec<Id> = self
            .ordered_ids(State::Focus)
            .into_iter()
            .filter(|&x| x != id)
            .collect();
        self.change_state(i, State::Focus, now);
        queue.insert(queue.len().min(1), id);
        self.set_order(&queue);
        self.renumber();
        Ok(())
    }

    /// "Verschieben": move a focus item by `delta` places, only within #2 … n.
    pub fn move_by(&mut self, id: Id, delta: i32) -> Result<(), Error> {
        let i = self.index(id)?;
        let it = &self.items[i];
        if it.state != State::Focus {
            return Err(Error::NotApplicable);
        }
        if delta == 0 {
            return Err(Error::Unchanged);
        }
        let mut queue = self.ordered_ids(State::Focus);
        let from = it.order as i64;
        let to = from + i64::from(delta);
        if from < 1 || to < 1 || to >= queue.len() as i64 {
            return Err(Error::NotApplicable);
        }
        let moved = queue.remove(from as usize);
        queue.insert(to as usize, moved);
        self.set_order(&queue);
        Ok(())
    }

    /// "Erledigt": `done` with `done_at`. If it was #1, the next one moves up as "Bereit" (not
    /// active, no auto-start).
    pub fn finish(&mut self, id: Id, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        if self.items[i].state == State::Done {
            return Err(Error::Unchanged);
        }
        self.change_state(i, State::Done, now);
        self.items[i].done_at = Some(now);
        self.renumber();
        Ok(())
    }

    /// "Delegieren": `delegated` with `delegated_to` and `delegated_at`. Delegating a delegated
    /// item to someone else updates both.
    pub fn delegate(&mut self, id: Id, to: &str, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let name = person_name(to)?;
        let it = &self.items[i];
        if it.state == State::Delegated && it.delegated_to.as_deref() == Some(name.as_str()) {
            return Err(Error::Unchanged);
        }
        self.change_state(i, State::Delegated, now);
        let it = &mut self.items[i];
        it.delegated_to = Some(name);
        it.delegated_at = Some(now);
        it.updated_at = now;
        self.renumber();
        Ok(())
    }

    /// "Zurück in Fokus", "→ Fokus", "⚡ Zw.", "→ Idee": to the end of the given list.
    pub fn send_to(&mut self, id: Id, list: List, now: DateTime<Utc>) -> Result<(), Error> {
        let i = self.index(id)?;
        let state = State::from(list);
        if self.items[i].state == state {
            return Err(Error::Unchanged);
        }
        self.change_state(i, state, now);
        self.renumber();
        Ok(())
    }

    /// Start the coffee break; needs an active task.
    pub fn pause(&mut self) -> Result<(), Error> {
        if self.on_break {
            return Err(Error::Unchanged);
        }
        if self.active().is_none() {
            return Err(Error::NoActiveItem);
        }
        self.on_break = true;
        Ok(())
    }

    /// End the coffee break; the same task goes on.
    pub fn resume(&mut self) -> Result<(), Error> {
        if !self.on_break {
            return Err(Error::Unchanged);
        }
        self.on_break = false;
        Ok(())
    }

    /// Check the invariants of `docs/KONZEPT.md` 2.3, plus unique ids and `order` 0 outside the
    /// ordered lists.
    pub fn check_invariants(&self) -> Result<(), InvariantViolation> {
        let fail = |msg: String| Err(InvariantViolation(msg));
        let mut ids = HashSet::new();
        for it in &self.items {
            if !ids.insert(it.id) {
                return fail(format!("duplicate id {}", it.id));
            }
            if (it.active || it.last_active) && it.state != State::Focus {
                return fail(format!("{} is active or last active outside focus", it.id));
            }
            if it.active && it.order != 0 {
                return fail(format!("active item {} has order {}", it.id, it.order));
            }
            if !it.state.is_ordered() && it.order != 0 {
                return fail(format!("unordered item {} has order {}", it.id, it.order));
            }
        }
        let active = self.items.iter().filter(|it| it.active).count();
        if active > 1 {
            return fail(format!("{active} active items"));
        }
        let last_active = self.items.iter().filter(|it| it.last_active).count();
        if last_active > 1 {
            return fail(format!("{last_active} last active items"));
        }
        for state in [State::Focus, State::Intermezzo] {
            let mut orders: Vec<u32> = self
                .items
                .iter()
                .filter(|it| it.state == state)
                .map(|it| it.order)
                .collect();
            orders.sort_unstable();
            if orders.iter().enumerate().any(|(i, &o)| o as usize != i) {
                return fail(format!(
                    "order of {state:?} is not gap-free from 0: {orders:?}"
                ));
            }
        }
        if self.on_break && active == 0 {
            return fail("on break without an active item".into());
        }
        Ok(())
    }

    fn index(&self, id: Id) -> Result<usize, Error> {
        self.items
            .iter()
            .position(|it| it.id == id)
            .ok_or(Error::UnknownItem(id))
    }

    fn item(&self, id: Id) -> &Item {
        self.get(id).expect("id taken from the store")
    }

    fn ordered_ids(&self, state: State) -> Vec<Id> {
        self.list(state).into_iter().map(|it| it.id).collect()
    }

    /// Give the items `order` 0, 1, … in the order of `ids`.
    fn set_order(&mut self, ids: &[Id]) {
        for it in &mut self.items {
            if let Some(pos) = ids.iter().position(|&id| id == it.id) {
                it.order = pos as u32;
            }
        }
    }

    /// Close the gaps in focus and intermezzo, keeping their order.
    fn renumber(&mut self) {
        for state in [State::Focus, State::Intermezzo] {
            let ids = self.ordered_ids(state);
            self.set_order(&ids);
        }
    }

    /// Move an item into another state and clean up what belonged to the old one. A new place in
    /// an ordered list is at the end until the caller renumbers or sets the order.
    fn change_state(&mut self, i: usize, state: State, now: DateTime<Utc>) {
        let it = &mut self.items[i];
        if it.state == state {
            return;
        }
        let was_active = it.active;
        if it.title.trim().is_empty() && state != State::Idea {
            it.title = title_from_note(&it.note);
        }
        match it.state {
            State::Focus => {
                it.active = false;
                it.last_active = false;
            }
            State::Delegated => {
                it.delegated_to = None;
                it.delegated_at = None;
            }
            State::Done => it.done_at = None,
            State::Idea | State::Intermezzo => {}
        }
        it.state = state;
        it.order = if state.is_ordered() { u32::MAX } else { 0 };
        it.updated_at = now;
        if was_active {
            self.on_break = false;
        }
    }
}

/// A person's name from `@Name` or `Name`, refused if empty.
fn person_name(raw: &str) -> Result<String, Error> {
    let name = raw.trim().trim_start_matches('@').trim();
    if name.is_empty() {
        Err(Error::EmptyName)
    } else {
        Ok(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 24, 9, minute, 0).unwrap()
    }

    fn add(store: &mut Store, title: &str, target: Target) -> Id {
        let new = NewItem::new(title);
        let id = new.id;
        store.add(new, target, t(0)).unwrap();
        id
    }

    /// Titles of the focus list in order, with `*` for active and `↩` for last active.
    fn focus(store: &Store) -> Vec<String> {
        store
            .list(State::Focus)
            .iter()
            .map(|it| {
                let mark = if it.active {
                    "*"
                } else if it.last_active {
                    "↩"
                } else {
                    ""
                };
                format!("{}{mark}", it.title)
            })
            .collect()
    }

    /// A store with focus `a* b c`, intermezzo `z`, idea `i`.
    fn sample() -> (Store, [Id; 5]) {
        let mut s = Store::new();
        let a = add(&mut s, "a", Target::Now);
        let b = add(&mut s, "b", Target::Focus);
        let c = add(&mut s, "c", Target::Focus);
        let z = add(&mut s, "z", Target::Intermezzo);
        let i = add(&mut s, "i", Target::Idea);
        assert_eq!(focus(&s), ["a*", "b", "c"]);
        (s, [a, b, c, z, i])
    }

    fn ok(store: &Store) {
        store.check_invariants().unwrap();
    }

    // --- Capture (KONZEPT 3.1) ---

    #[test]
    fn add_now_takes_over_and_marks_the_left_task() {
        let (mut s, _) = sample();
        add(&mut s, "n", Target::Now);
        assert_eq!(focus(&s), ["n*", "a↩", "b", "c"]);
        ok(&s);
    }

    #[test]
    fn add_to_intermezzo_and_delegated() {
        let (mut s, _) = sample();
        let y = add(&mut s, "y", Target::Intermezzo);
        assert_eq!(s.item(y).order, 1);
        let d = add(&mut s, "d", Target::Delegated(" @Jonas ".into()));
        assert_eq!(s.item(d).delegated_to.as_deref(), Some("Jonas"));
        assert_eq!(s.item(d).delegated_at, Some(t(0)));
        ok(&s);
    }

    #[test]
    fn add_refuses_duplicate_id_and_empty_delegate() {
        let (mut s, [a, ..]) = sample();
        let before = s.clone();
        let mut dup = NewItem::new("x");
        dup.id = a;
        assert_eq!(s.add(dup, Target::Idea, t(1)), Err(Error::DuplicateId(a)));
        assert_eq!(
            s.add(NewItem::new("x"), Target::Delegated("@ ".into()), t(1)),
            Err(Error::EmptyName)
        );
        assert_eq!(s, before);
    }

    // --- Übernehmen ---

    #[test]
    fn start_from_queue_moves_old_top_to_second_with_last_active() {
        let (mut s, [_, _, c, ..]) = sample();
        s.start(c, t(1)).unwrap();
        assert_eq!(focus(&s), ["c*", "a↩", "b"]);
        ok(&s);
    }

    #[test]
    fn start_last_active_again_clears_the_mark_and_marks_the_other() {
        let (mut s, [a, _, c, ..]) = sample();
        s.start(c, t(1)).unwrap();
        s.start(a, t(2)).unwrap();
        assert_eq!(focus(&s), ["a*", "c↩", "b"]);
        ok(&s);
    }

    #[test]
    fn start_from_intermezzo_and_idea_makes_focus() {
        let (mut s, [_, _, _, z, i]) = sample();
        s.start(z, t(1)).unwrap();
        assert_eq!(focus(&s), ["z*", "a↩", "b", "c"]);
        assert!(s.list(State::Intermezzo).is_empty());
        s.start(i, t(2)).unwrap();
        assert_eq!(focus(&s), ["i*", "z↩", "a", "b", "c"]);
        ok(&s);
    }

    #[test]
    fn start_when_ready_moves_old_top_down_without_mark() {
        let (mut s, [a, b, c, ..]) = sample();
        s.start(c, t(1)).unwrap(); // c* a↩ b
        s.finish(c, t(2)).unwrap(); // a↩ (ready) b
        assert!(s.active().is_none());
        s.start(b, t(3)).unwrap();
        // a was not active, so it gets no new mark; it keeps the one it had.
        assert_eq!(focus(&s), ["b*", "a↩"]);
        s.finish(b, t(4)).unwrap();
        s.start(a, t(5)).unwrap();
        assert_eq!(focus(&s), ["a*"]);
        ok(&s);
    }

    #[test]
    fn start_the_active_task_is_unchanged_or_resumes_the_break() {
        let (mut s, [a, ..]) = sample();
        assert_eq!(s.start(a, t(1)), Err(Error::Unchanged));
        s.pause().unwrap();
        s.start(a, t(2)).unwrap();
        assert!(!s.on_break());
        assert_eq!(focus(&s), ["a*", "b", "c"]);
    }

    #[test]
    fn start_another_task_ends_the_break() {
        let (mut s, [_, b, ..]) = sample();
        s.pause().unwrap();
        s.start(b, t(1)).unwrap();
        assert!(!s.on_break());
        assert_eq!(focus(&s), ["b*", "a↩", "c"]);
        ok(&s);
    }

    #[test]
    fn start_unknown_item() {
        let (mut s, _) = sample();
        let id = Id::generate();
        assert_eq!(s.start(id, t(1)), Err(Error::UnknownItem(id)));
    }

    // --- Als Nächstes ---

    #[test]
    fn to_next_puts_item_on_second_and_keeps_current_work() {
        let (mut s, [_, _, c, z, i]) = sample();
        s.to_next(c, t(1)).unwrap();
        assert_eq!(focus(&s), ["a*", "c", "b"]);
        s.to_next(z, t(2)).unwrap();
        assert_eq!(focus(&s), ["a*", "z", "c", "b"]);
        s.to_next(i, t(3)).unwrap();
        assert_eq!(focus(&s), ["a*", "i", "z", "c", "b"]);
        ok(&s);
    }

    #[test]
    fn taking_from_intermezzo_closes_the_gap_there() {
        let (mut s, [.., z, _]) = sample();
        let y = add(&mut s, "y", Target::Intermezzo);
        let x = add(&mut s, "x", Target::Intermezzo);
        s.to_next(z, t(1)).unwrap();
        ok(&s);
        s.start(y, t(2)).unwrap();
        ok(&s);
        assert_eq!(s.item(x).order, 0);
    }

    #[test]
    fn to_next_on_first_or_second_does_not_apply() {
        let (mut s, [a, b, ..]) = sample();
        assert_eq!(s.to_next(a, t(1)), Err(Error::NotApplicable));
        assert_eq!(s.to_next(b, t(1)), Err(Error::Unchanged));
    }

    #[test]
    fn to_next_into_empty_focus_is_ready_on_first() {
        let mut s = Store::new();
        let i = add(&mut s, "i", Target::Idea);
        s.to_next(i, t(1)).unwrap();
        assert_eq!(focus(&s), ["i"]);
        ok(&s);
    }

    // --- Verschieben ---

    #[test]
    fn move_by_only_within_second_to_last() {
        let (mut s, [a, b, c, z, _]) = sample();
        s.move_by(b, 1).unwrap();
        assert_eq!(focus(&s), ["a*", "c", "b"]);
        assert_eq!(s.move_by(c, -1), Err(Error::NotApplicable));
        assert_eq!(s.move_by(b, 1), Err(Error::NotApplicable));
        assert_eq!(s.move_by(a, 1), Err(Error::NotApplicable));
        assert_eq!(s.move_by(z, 1), Err(Error::NotApplicable));
        assert_eq!(s.move_by(b, 0), Err(Error::Unchanged));
        ok(&s);
    }

    // --- Erledigt ---

    #[test]
    fn finish_first_makes_second_ready_without_auto_start() {
        let (mut s, [a, ..]) = sample();
        s.finish(a, t(5)).unwrap();
        assert_eq!(focus(&s), ["b", "c"]);
        assert!(s.active().is_none());
        let done = s.item(a);
        assert_eq!(done.state, State::Done);
        assert_eq!(done.done_at, Some(t(5)));
        assert!(!done.active && !done.last_active);
        ok(&s);
    }

    #[test]
    fn finish_active_during_break_ends_the_break() {
        let (mut s, [a, ..]) = sample();
        s.pause().unwrap();
        s.finish(a, t(1)).unwrap();
        assert!(!s.on_break());
        ok(&s);
    }

    #[test]
    fn finish_from_queue_intermezzo_idea_and_twice() {
        let (mut s, [_, b, _, z, i]) = sample();
        s.finish(b, t(1)).unwrap();
        s.finish(z, t(1)).unwrap();
        s.finish(i, t(1)).unwrap();
        assert_eq!(focus(&s), ["a*", "c"]);
        assert_eq!(s.finish(b, t(2)), Err(Error::Unchanged));
        ok(&s);
    }

    // --- Delegieren ---

    #[test]
    fn delegate_sets_person_and_time() {
        let (mut s, [a, ..]) = sample();
        s.delegate(a, "@Jonas", t(3)).unwrap();
        let it = s.item(a);
        assert_eq!(it.state, State::Delegated);
        assert_eq!(it.delegated_to.as_deref(), Some("Jonas"));
        assert_eq!(it.delegated_at, Some(t(3)));
        assert_eq!(focus(&s), ["b", "c"]);
        assert_eq!(s.delegate(a, "Jonas", t(4)), Err(Error::Unchanged));
        s.delegate(a, "Buchhaltung", t(5)).unwrap();
        assert_eq!(s.item(a).delegated_to.as_deref(), Some("Buchhaltung"));
        assert_eq!(s.item(a).delegated_at, Some(t(5)));
        assert_eq!(s.delegate(a, "  @ ", t(6)), Err(Error::EmptyName));
        ok(&s);
    }

    // --- Zurück in Fokus, → Fokus, ⚡ Zw. ---

    #[test]
    fn back_to_focus_goes_to_the_end_and_forgets_the_delegation() {
        let (mut s, [_, b, ..]) = sample();
        s.delegate(b, "Jonas", t(1)).unwrap();
        s.send_to(b, List::Focus, t(2)).unwrap();
        assert_eq!(focus(&s), ["a*", "c", "b"]);
        assert_eq!(s.item(b).delegated_to, None);
        assert_eq!(s.item(b).delegated_at, None);
        ok(&s);
    }

    #[test]
    fn promote_idea_to_the_end_of_focus_or_intermezzo() {
        let (mut s, [.., i]) = sample();
        let j = add(&mut s, "j", Target::Idea);
        s.send_to(i, List::Focus, t(1)).unwrap();
        assert_eq!(focus(&s), ["a*", "b", "c", "i"]);
        s.send_to(j, List::Intermezzo, t(1)).unwrap();
        let inter: Vec<_> = s
            .list(State::Intermezzo)
            .iter()
            .map(|it| it.title.as_str())
            .collect();
        assert_eq!(inter, ["z", "j"]);
        assert_eq!(s.send_to(j, List::Intermezzo, t(2)), Err(Error::Unchanged));
        ok(&s);
    }

    #[test]
    fn promote_without_title_takes_the_first_note_line() {
        let mut s = Store::new();
        let mut new = NewItem::new("");
        new.note = "\n- Farben aus dem System-Theme\n- Kontrast".into();
        let id = new.id;
        s.add(new, Target::Idea, t(0)).unwrap();
        assert_eq!(s.item(id).title, "");
        s.send_to(id, List::Focus, t(1)).unwrap();
        assert_eq!(s.item(id).title, "Farben aus dem System-Theme");
    }

    #[test]
    fn sending_the_active_task_away_clears_flags_and_break() {
        let (mut s, [a, b, ..]) = sample();
        s.start(b, t(1)).unwrap(); // b* a↩ c
        s.pause().unwrap();
        s.send_to(b, List::Idea, t(2)).unwrap();
        assert!(!s.on_break());
        assert_eq!(focus(&s), ["a↩", "c"]);
        s.send_to(a, List::Intermezzo, t(3)).unwrap();
        assert_eq!(focus(&s), ["c"]);
        assert!(!s.item(a).last_active);
        ok(&s);
    }

    #[test]
    fn reopening_a_done_item_clears_done_at() {
        let (mut s, [_, b, ..]) = sample();
        s.finish(b, t(1)).unwrap();
        s.send_to(b, List::Focus, t(2)).unwrap();
        assert_eq!(s.item(b).done_at, None);
        assert_eq!(focus(&s), ["a*", "c", "b"]);
        ok(&s);
    }

    // --- Pause / Weiter ---

    #[test]
    fn pause_needs_an_active_task_and_resume_continues() {
        let (mut s, [a, ..]) = sample();
        s.pause().unwrap();
        assert!(s.on_break());
        assert_eq!(s.pause(), Err(Error::Unchanged));
        s.resume().unwrap();
        assert!(!s.on_break());
        assert_eq!(s.resume(), Err(Error::Unchanged));
        assert_eq!(s.active().map(|it| it.id), Some(a));
        s.finish(a, t(1)).unwrap();
        assert_eq!(s.pause(), Err(Error::NoActiveItem));
    }

    // --- Invariants ---

    #[test]
    fn from_parts_refuses_broken_data() {
        let (s, [_, b, ..]) = sample();
        let mut items = s.items().to_vec();
        items.iter_mut().find(|it| it.id == b).unwrap().active = true;
        assert!(Store::from_parts(false, items).is_err());
        let (s, [a, ..]) = sample();
        let mut items = s.items().to_vec();
        items.retain(|it| it.id != a);
        assert!(
            Store::from_parts(false, items.clone()).is_err(),
            "gap at order 0"
        );
        assert!(
            Store::from_parts(true, Vec::new()).is_err(),
            "break without active"
        );
    }

    #[test]
    fn to_back_puts_the_next_one_on_ready() {
        let mut s = Store::new();
        let a = add(&mut s, "a", Target::Now);
        add(&mut s, "b", Target::Focus);
        s.pause().unwrap();
        s.to_back(a, t(2)).unwrap();
        assert_eq!(focus(&s), ["b", "a"]);
        assert!(s.active().is_none() && !s.on_break());
        assert_eq!(s.to_back(a, t(3)), Err(Error::Unchanged));
        let z = add(&mut s, "z", Target::Idea);
        assert_eq!(s.to_back(z, t(3)), Err(Error::NotApplicable));
        s.check_invariants().unwrap();
    }

    #[test]
    fn card_edits_change_one_item() {
        let mut s = Store::new();
        let a = add(&mut s, "a", Target::Now);
        s.rename(a, "  new title ", t(1)).unwrap();
        assert_eq!(s.get(a).unwrap().title, "new title");
        assert_eq!(s.rename(a, "new title", t(2)), Err(Error::Unchanged));
        assert_eq!(s.rename(a, "  ", t(2)), Err(Error::EmptyName));
        s.set_note(a, "line".into(), t(3)).unwrap();
        s.add_check(a, "step", t(4)).unwrap();
        assert_eq!(s.add_check(a, " ", t(4)), Err(Error::EmptyName));
        let step = s.get(a).unwrap().checklist[0].id;
        s.toggle_check(a, step, t(5)).unwrap();
        assert!(s.get(a).unwrap().checklist[0].done);
        s.toggle_check(a, step, t(6)).unwrap();
        assert!(!s.get(a).unwrap().checklist[0].done);
        assert!(matches!(
            s.toggle_check(a, Id::generate(), t(6)),
            Err(Error::UnknownItem(_))
        ));
        s.check_invariants().unwrap();
    }

    #[test]
    fn brainstorm_thoughts_grow_the_note_and_links_become_resources() {
        let mut s = Store::new();
        let a = add(&mut s, "Idee", Target::Idea);
        s.brainstorm(a, "  erster Gedanke ", t(1)).unwrap();
        s.brainstorm(a, "zweiter", t(2)).unwrap();
        assert_eq!(s.get(a).unwrap().note, "- erster Gedanke\n- zweiter");
        s.brainstorm(a, "https://example.com/x", t(3)).unwrap();
        assert_eq!(s.get(a).unwrap().resources.len(), 1);
        assert_eq!(s.get(a).unwrap().note, "- erster Gedanke\n- zweiter");
        assert_eq!(s.brainstorm(a, "  ", t(4)), Err(Error::EmptyName));
        // An idea without a title takes its first bullet as title when it goes to focus.
        let mut new = NewItem::new("");
        new.note = String::new();
        let blank = new.id;
        s.add(new, Target::Idea, t(5)).unwrap();
        s.brainstorm(blank, "Stichwort", t(6)).unwrap();
        s.send_to(blank, List::Focus, t(7)).unwrap();
        assert_eq!(s.get(blank).unwrap().title, "Stichwort");
        s.check_invariants().unwrap();
    }

    #[test]
    fn opening_a_resource_ranks_it_first_and_removing_takes_it_off() {
        let mut s = Store::new();
        let a = add(&mut s, "a", Target::Now);
        s.add_resource(a, "https://example.com/1", t(1)).unwrap();
        s.add_resource(a, "https://example.com/2", t(1)).unwrap();
        let second = s.get(a).unwrap().resources[1].id;
        s.mark_used(a, second, t(5)).unwrap();
        let ranked = crate::resources::ranked(&s.get(a).unwrap().resources);
        assert_eq!(ranked[0].resource.id, second);
        // The digit stays with the order of adding.
        assert_eq!(ranked[0].digit, Some(2));
        s.remove_resource(a, second, t(6)).unwrap();
        assert_eq!(s.get(a).unwrap().resources.len(), 1);
        assert!(matches!(
            s.remove_resource(a, second, t(7)),
            Err(Error::UnknownItem(_))
        ));
        assert!(matches!(
            s.mark_used(a, second, t(7)),
            Err(Error::UnknownItem(_))
        ));
    }

    #[test]
    fn adding_a_resource_needs_a_link_path_or_mail() {
        let mut s = Store::new();
        let a = add(&mut s, "a", Target::Now);
        s.add_resource(a, " https://example.com/x ", t(1)).unwrap();
        assert_eq!(
            s.get(a).unwrap().resources[0].target,
            "https://example.com/x"
        );
        assert_eq!(
            s.add_resource(a, "https://example.com/x", t(2)),
            Err(Error::Unchanged)
        );
        assert_eq!(
            s.add_resource(a, "just words", t(2)),
            Err(Error::NotApplicable)
        );
    }

    // --- Ausarbeitung: Datum und Schritte bearbeiten ---

    #[test]
    fn date_is_noted_and_cleared_without_other_effect() {
        let (mut s, [a, b, ..]) = sample();
        let day = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        s.set_date(b, Some(day), t(3)).unwrap();
        assert_eq!(s.item(b).date, Some(day));
        assert_eq!(s.item(b).state, State::Focus);
        assert_eq!(s.set_date(b, Some(day), t(4)), Err(Error::Unchanged));
        s.set_date(b, None, t(5)).unwrap();
        assert_eq!(s.item(b).date, None);
        assert_eq!(s.set_date(a, None, t(6)), Err(Error::Unchanged));
        ok(&s);
    }

    #[test]
    fn steps_can_be_reworded_and_removed() {
        let (mut s, [a, ..]) = sample();
        s.add_check(a, "eins", t(1)).unwrap();
        s.add_check(a, "zwei", t(2)).unwrap();
        let first = s.item(a).checklist[0].id;
        let second = s.item(a).checklist[1].id;
        s.rename_check(a, first, "  erstens ", t(3)).unwrap();
        assert_eq!(s.item(a).checklist[0].text, "erstens");
        assert_eq!(
            s.rename_check(a, first, "erstens", t(4)),
            Err(Error::Unchanged)
        );
        assert_eq!(s.rename_check(a, first, "  ", t(4)), Err(Error::EmptyName));
        s.remove_check(a, first, t(5)).unwrap();
        assert_eq!(s.item(a).checklist.len(), 1);
        assert_eq!(s.item(a).checklist[0].id, second);
        assert_eq!(
            s.remove_check(a, first, t(6)),
            Err(Error::UnknownItem(first))
        );
        ok(&s);
    }
}
