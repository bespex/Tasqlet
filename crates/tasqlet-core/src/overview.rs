//! The state at a glance, for the tray icon, its tooltip and the first menu line
//! (`docs/KONZEPT.md` 4). Texts are made by the shell, which knows the language.

use crate::model::{Item, State};
use crate::store::Store;

/// What the user is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Nothing in focus.
    Idle,
    /// Focus items, but none active ("Bereit", e.g. after finishing one).
    Ready,
    /// A task is active: the icon shows a dot.
    Working,
    /// Coffee break: the icon shows a cup.
    Break,
}

/// The state at a glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overview<'a> {
    pub mode: Mode,
    /// Number of focus items: the number on the icon.
    pub focus_count: usize,
    /// #1 of the focus list (active or ready).
    pub current: Option<&'a Item>,
    /// #2 of the focus list: "Danach".
    pub next: Option<&'a Item>,
    /// Checked and total checklist entries of `current`, if it has a checklist.
    pub progress: Option<(usize, usize)>,
}

impl Store {
    /// The state at a glance.
    pub fn overview(&self) -> Overview<'_> {
        let focus = self.list(State::Focus);
        let current = focus.first().copied();
        let mode = match current {
            None => Mode::Idle,
            Some(it) if it.active && self.on_break() => Mode::Break,
            Some(it) if it.active => Mode::Working,
            Some(_) => Mode::Ready,
        };
        let progress = current.filter(|it| !it.checklist.is_empty()).map(|it| {
            let done = it.checklist.iter().filter(|c| c.done).count();
            (done, it.checklist.len())
        });
        Overview {
            mode,
            focus_count: focus.len(),
            current,
            next: focus.get(1).copied(),
            progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewItem, Target};
    use chrono::Utc;

    fn add(s: &mut Store, title: &str, target: Target, steps: usize) -> crate::Id {
        let mut new = NewItem::new(title);
        new.checklist = (0..steps).map(|i| format!("Schritt {i}")).collect();
        let id = new.id;
        s.add(new, target, Utc::now()).unwrap();
        id
    }

    #[test]
    fn modes_and_numbers() {
        let mut s = Store::new();
        assert_eq!(s.overview().mode, Mode::Idle);
        assert_eq!(s.overview().focus_count, 0);

        let a = add(&mut s, "a", Target::Now, 3);
        add(&mut s, "b", Target::Focus, 0);
        add(&mut s, "z", Target::Intermezzo, 0);
        let o = s.overview();
        assert_eq!(o.mode, Mode::Working);
        assert_eq!(o.focus_count, 2);
        assert_eq!(o.current.map(|it| it.title.as_str()), Some("a"));
        assert_eq!(o.next.map(|it| it.title.as_str()), Some("b"));
        assert_eq!(o.progress, Some((0, 3)));

        s.pause().unwrap();
        assert_eq!(s.overview().mode, Mode::Break);

        s.finish(a, Utc::now()).unwrap();
        let o = s.overview();
        assert_eq!(o.mode, Mode::Ready);
        assert_eq!(o.current.map(|it| it.title.as_str()), Some("b"));
        assert_eq!(o.next, None);
        assert_eq!(o.progress, None);
    }
}
