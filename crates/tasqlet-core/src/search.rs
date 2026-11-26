//! Full text search over everything in memory, open and finished (`docs/KONZEPT.md` 6.4, plan 1.10).
//!
//! All items, also the archive, are loaded at start (D21), so searching is a scan over the store.
//! Every word of the query has to occur somewhere in an item (title, note, checklist, label or target
//! of a resource, the person it is delegated to, tags); case does not matter. Better places weigh
//! more, so a title hit comes before a note hit; open items come before finished ones.

use std::cmp::Reverse;

use crate::model::{Item, State};
use crate::store::Store;

/// The words of a query: split at whitespace, lower case. Also what the interface marks.
pub fn terms(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

const TITLE: u32 = 100;
const PERSON: u32 = 60;
const TAG: u32 = 50;
const RESOURCE: u32 = 40;
const STEP: u32 = 30;
const NOTE: u32 = 20;

/// How well `item` matches `terms` (all of them), `None` if one is missing.
fn score(item: &Item, terms: &[String]) -> Option<u32> {
    let title = item.effective_title().to_lowercase();
    let person = item.delegated_to.as_deref().unwrap_or("").to_lowercase();
    let note = item.note.to_lowercase();
    let mut total = 0;
    for term in terms {
        let term = term.trim_start_matches('@');
        if term.is_empty() {
            continue;
        }
        let best = [
            (title.contains(term), TITLE),
            (person.contains(term), PERSON),
            (
                item.tags.iter().any(|t| t.to_lowercase().contains(term)),
                TAG,
            ),
            (
                item.resources.iter().any(|r| {
                    r.target.to_lowercase().contains(term)
                        || r.shown_label().to_lowercase().contains(term)
                }),
                RESOURCE,
            ),
            (
                item.checklist
                    .iter()
                    .any(|c| c.text.to_lowercase().contains(term)),
                STEP,
            ),
            (note.contains(term), NOTE),
        ]
        .into_iter()
        .filter(|(hit, _)| *hit)
        .map(|(_, weight)| weight)
        .max()?;
        total += best;
    }
    Some(total)
}

/// The items that match `query`, best first, at most `limit`. A blank query matches nothing.
pub fn search<'a>(store: &'a Store, query: &str, limit: usize) -> Vec<&'a Item> {
    let terms = terms(query);
    if terms.iter().all(|t| t.trim_start_matches('@').is_empty()) {
        return Vec::new();
    }
    let mut hits: Vec<(u32, &Item)> = store
        .items()
        .iter()
        .map(|item| &**item)
        .filter_map(|item| score(item, &terms).map(|s| (s, item)))
        .collect();
    hits.sort_by_key(|(score, item)| {
        (
            Reverse(*score),
            item.state == State::Done,
            Reverse(item.updated_at),
            item.id,
        )
    });
    hits.into_iter().take(limit).map(|(_, item)| item).collect()
}

/// What the search shows without a query: the finished items, latest first.
pub fn recently_done(store: &Store, limit: usize) -> Vec<&Item> {
    let mut done: Vec<&Item> = store
        .items()
        .iter()
        .map(|it| &**it)
        .filter(|it| it.state == State::Done)
        .collect();
    done.sort_by_key(|it| (Reverse(it.done_at), it.id));
    done.truncate(limit);
    done
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewItem, Target};
    use chrono::{DateTime, Utc};

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_795_000_000 + minute * 60, 0).unwrap()
    }

    fn add(store: &mut Store, title: &str, note: &str, target: Target, minute: i64) -> crate::Id {
        let mut new = NewItem::new(title);
        new.note = note.into();
        let id = new.id;
        store.add(new, target, at(minute)).unwrap();
        id
    }

    fn titles(hits: &[&Item]) -> Vec<String> {
        hits.iter().map(|i| i.title.clone()).collect()
    }

    #[test]
    fn every_word_has_to_occur_and_case_does_not_matter() {
        let mut s = Store::new();
        add(&mut s, "Webhook umstellen", "", Target::Focus, 1);
        add(
            &mut s,
            "Release Notes",
            "Webhook erwähnen",
            Target::Focus,
            2,
        );
        add(&mut s, "Anderes", "", Target::Idea, 3);
        assert_eq!(titles(&search(&s, "WEBHOOK", 10)).len(), 2);
        assert_eq!(titles(&search(&s, "webhook notes", 10)), ["Release Notes"]);
        assert!(search(&s, "webhook gibtsnicht", 10).is_empty());
        assert!(search(&s, "   ", 10).is_empty());
        assert!(search(&s, "@", 10).is_empty());
    }

    #[test]
    fn a_title_hit_comes_before_a_note_hit_and_open_before_done() {
        let mut s = Store::new();
        let note = add(&mut s, "Notiz", "Jira klären", Target::Focus, 1);
        let title = add(&mut s, "Jira klären", "", Target::Focus, 2);
        let done = add(&mut s, "Jira erledigt", "", Target::Focus, 3);
        s.finish(done, at(4)).unwrap();
        let hits = search(&s, "jira", 10);
        assert_eq!(
            hits.iter().map(|i| i.id).collect::<Vec<_>>(),
            vec![title, done, note]
        );
    }

    #[test]
    fn it_looks_into_steps_resources_and_the_delegate_and_finds_finished_items() {
        let mut s = Store::new();
        let a = add(&mut s, "Aufgabe", "", Target::Focus, 1);
        s.add_check(a, "Staging prüfen", at(2)).unwrap();
        s.add_resource(a, "https://jira.example.com/browse/CORE-418", at(2))
            .unwrap();
        let d = add(
            &mut s,
            "Übergeben",
            "",
            Target::Delegated("@Jonas".into()),
            3,
        );
        let old = add(&mut s, "Altes", "", Target::Focus, 4);
        s.finish(old, at(5)).unwrap();
        assert_eq!(search(&s, "staging", 10)[0].id, a);
        assert_eq!(search(&s, "CORE-418", 10)[0].id, a);
        assert_eq!(search(&s, "@jonas", 10)[0].id, d);
        assert_eq!(search(&s, "jonas", 10)[0].id, d);
        assert_eq!(search(&s, "altes", 10)[0].id, old);
    }

    #[test]
    fn limit_and_recently_done() {
        let mut s = Store::new();
        let ids: Vec<_> = (0..5)
            .map(|i| add(&mut s, &format!("Task {i}"), "", Target::Focus, i))
            .collect();
        for (n, id) in ids.iter().enumerate() {
            s.finish(*id, at(10 + n as i64)).unwrap();
        }
        assert_eq!(search(&s, "task", 3).len(), 3);
        let recent = recently_done(&s, 2);
        assert_eq!(titles(&recent), ["Task 4", "Task 3"]);
    }

    #[test]
    fn five_thousand_items_answer_quickly() {
        let mut s = Store::new();
        for i in 0..5000 {
            let mut new = NewItem::new(format!("Eintrag {i} zum Thema {}", i % 37));
            new.note = format!("Notiz mit etwas Text {i} und noch mehr Wörtern darin");
            new.checklist = vec![format!("Schritt {i}")];
            let id = new.id;
            s.add(new, Target::Idea, at(i)).unwrap();
            if i % 2 == 0 {
                s.finish(id, at(i + 1)).unwrap();
            }
        }
        let start = std::time::Instant::now();
        let hits = search(&s, "thema 12 notiz", 50);
        let took = start.elapsed();
        assert!(!hits.is_empty());
        // Release builds are far faster; even unoptimised this stays well below the limit of 50 ms.
        assert!(took.as_millis() < 50, "search took {took:?}");
    }
}
