//! Property test: no sequence of actions, undos included, breaks an invariant of
//! `docs/KONZEPT.md` 2.3, and a refused action changes nothing.

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use tasqlet_core::{Action, Id, List, NewItem, Session, State, Target};

/// An action with item references as indices, resolved against the items present at that moment.
#[derive(Debug, Clone)]
enum Step {
    Add {
        title: String,
        note: String,
        target: u8,
    },
    Start(usize),
    ToNext(usize),
    MoveBy(usize, i32),
    Finish(usize),
    Delegate(usize, String),
    SendTo(usize, u8),
    Pause,
    Resume,
    Undo,
    Unknown,
}

fn step() -> impl Strategy<Value = Step> {
    let idx = any::<usize>();
    prop_oneof![
        3 => ("[a-c]{0,2}", "(- )?[a-c ]{0,4}", 0u8..6)
            .prop_map(|(title, note, target)| Step::Add { title, note, target }),
        3 => idx.prop_map(Step::Start),
        2 => idx.prop_map(Step::ToNext),
        2 => (idx, -3i32..=3).prop_map(|(i, d)| Step::MoveBy(i, d)),
        2 => idx.prop_map(Step::Finish),
        1 => (idx, "@?[A-C ]{0,3}").prop_map(|(i, n)| Step::Delegate(i, n)),
        2 => (idx, 0u8..3).prop_map(|(i, l)| Step::SendTo(i, l)),
        1 => Just(Step::Pause),
        1 => Just(Step::Resume),
        1 => Just(Step::Undo),
        1 => Just(Step::Unknown),
    ]
}

fn resolve(step: Step, ids: &[Id]) -> Option<Action> {
    let pick = |i: usize| ids.get(i % ids.len().max(1)).copied();
    Some(match step {
        Step::Add {
            title,
            note,
            target,
        } => {
            let mut item = NewItem::new(title);
            item.note = note;
            let target = match target {
                0 => Target::Now,
                1 => Target::Focus,
                2 => Target::Intermezzo,
                3 => Target::Idea,
                4 => Target::Delegated("Jonas".into()),
                _ => Target::Delegated("@".into()),
            };
            Action::Add { item, target }
        }
        Step::Start(i) => Action::Start(pick(i)?),
        Step::ToNext(i) => Action::ToNext(pick(i)?),
        Step::MoveBy(i, delta) => Action::MoveBy {
            id: pick(i)?,
            delta,
        },
        Step::Finish(i) => Action::Finish(pick(i)?),
        Step::Delegate(i, to) => Action::Delegate { id: pick(i)?, to },
        Step::SendTo(i, l) => {
            let list = [List::Focus, List::Intermezzo, List::Idea][l as usize];
            Action::SendTo { id: pick(i)?, list }
        }
        Step::Pause => Action::Pause,
        Step::Resume => Action::Resume,
        Step::Unknown => Action::Start(Id::generate()),
        Step::Undo => return None,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn no_action_sequence_breaks_an_invariant(steps in prop::collection::vec(step(), 1..80)) {
        let start = DateTime::<Utc>::from_timestamp(1_795_000_000, 0).unwrap();
        let mut session = Session::default();
        for (n, step) in steps.into_iter().enumerate() {
            let now = start + Duration::minutes(n as i64);
            let ids: Vec<Id> = session.store().items().iter().map(|it| it.id).collect();
            let before = session.store().clone();
            match resolve(step, &ids) {
                Some(action) => {
                    if session.apply(action.clone(), now).is_err() {
                        prop_assert_eq!(session.store(), &before, "refused {:?} changed the store", action);
                    }
                }
                None => {
                    let _ = session.undo();
                }
            }
            let store = session.store();
            if let Err(violation) = store.check_invariants() {
                return Err(TestCaseError::fail(format!("{violation} after step {n}: {store:#?}")));
            }
            // The active task is always shown as #1.
            if let Some(active) = store.active() {
                prop_assert_eq!(store.list(State::Focus)[0].id, active.id);
            }
        }
    }
}
