//! Quick-entry parser (`docs/KONZEPT.md` 3.1): shortcuts typed into the input field.
//!
//! | typed | meaning |
//! |---|---|
//! | `@Name` | delegate to Name (the first one counts) |
//! | `!heute`, `!morgen`, `!übermorgen`, `!fr`, `!freitag`, `!28.11.`, `!28.11.2026`, `!2026-11-28` | note a date (the first one counts); English `!today`, `!tomorrow`, `!fri` work too |
//! | a link, path or mail target | becomes a resource and leaves the title |
//!
//! A word that only looks like a shortcut (`!wichtig`, `!31.02.`) stays in the title. A link
//! alone gives an item titled with its smart label (`PR #42`).

use chrono::{Datelike, Duration, NaiveDate};

use crate::model::{NewItem, NewResource, Target};
use crate::resources::{classify, smart_label};

/// What the quick entry recognised.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuickEntry {
    pub title: String,
    pub date: Option<NaiveDate>,
    /// Person from `@Name`, without the `@`.
    pub delegate_to: Option<String>,
    pub resources: Vec<NewResource>,
}

impl QuickEntry {
    /// Nothing to save.
    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.resources.is_empty()
    }

    /// Where the entry goes when saved with the key for `chosen`: with `@Name` always to
    /// "Delegiert", else to `chosen`.
    pub fn target(&self, chosen: Target) -> Target {
        match &self.delegate_to {
            Some(name) => Target::Delegated(name.clone()),
            None => chosen,
        }
    }

    /// The new item for [`crate::Store::add`].
    pub fn into_new_item(self) -> NewItem {
        let mut item = NewItem::new(self.title);
        item.date = self.date;
        item.resources = self.resources;
        item
    }
}

/// Parse the text of the quick entry. `today` resolves relative dates.
pub fn parse(input: &str, today: NaiveDate) -> QuickEntry {
    let mut entry = QuickEntry::default();
    let mut words: Vec<&str> = Vec::new();
    for word in input.split_whitespace() {
        if let Some(rest) = word.strip_prefix('@') {
            let name = person(rest);
            if entry.delegate_to.is_none() && !name.is_empty() {
                entry.delegate_to = Some(name.to_string());
                continue;
            }
        } else if let Some(rest) = word.strip_prefix('!') {
            if entry.date.is_none()
                && let Some(date) = date(rest.trim_end_matches([',', ';']), today)
            {
                entry.date = Some(date);
                continue;
            }
        } else {
            let core = word
                .trim_start_matches(['<', '(', '"'])
                .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '>', '"']);
            if let Some(kind) = classify(core) {
                entry.resources.push(NewResource {
                    kind,
                    target: core.to_string(),
                    label: None,
                });
                continue;
            }
        }
        words.push(word);
    }
    entry.title = words.join(" ");
    if entry.title.is_empty()
        && let Some(first) = entry.resources.first()
    {
        entry.title = smart_label(&first.target);
    }
    entry
}

/// The name in `@Name,`: letters, digits, `.`, `_`, `-`, without trailing punctuation.
fn person(rest: &str) -> &str {
    let end = rest
        .char_indices()
        .find(|(_, c)| !(c.is_alphanumeric() || matches!(c, '.' | '_' | '-')))
        .map_or(rest.len(), |(i, _)| i);
    rest[..end].trim_end_matches(['.', '_', '-'])
}

const WEEKDAYS: [[&str; 2]; 7] = [
    ["montag", "monday"],
    ["dienstag", "tuesday"],
    ["mittwoch", "wednesday"],
    ["donnerstag", "thursday"],
    ["freitag", "friday"],
    ["samstag", "saturday"],
    ["sonntag", "sunday"],
];

/// A date from the text after `!`.
pub fn date(word: &str, today: NaiveDate) -> Option<NaiveDate> {
    let w = word.to_lowercase();
    let days = |n: i64| today.checked_add_signed(Duration::days(n));
    match w.as_str() {
        "heute" | "today" => return Some(today),
        "morgen" | "tomorrow" => return days(1),
        "übermorgen" | "uebermorgen" => return days(2),
        _ => {}
    }
    if w.chars().count() >= 2 && w.chars().all(char::is_alphabetic) {
        let weekday = WEEKDAYS
            .iter()
            .position(|names| names.iter().any(|n| n.starts_with(&w)))?;
        let today_index = today.weekday().num_days_from_monday() as i64;
        return days((weekday as i64 - today_index).rem_euclid(7));
    }
    if let Ok(iso) = NaiveDate::parse_from_str(&w, "%Y-%m-%d") {
        return Some(iso);
    }
    numeric_date(&w, today)
}

/// `28.11.`, `28.11`, `28.11.26`, `28.11.2026`. Without a year the next such day from today on.
fn numeric_date(w: &str, today: NaiveDate) -> Option<NaiveDate> {
    let parts: Vec<&str> = w.split('.').collect();
    let (d, m, y) = match parts.as_slice() {
        [d, m] | [d, m, ""] => (*d, *m, None),
        [d, m, y] if !y.is_empty() => (*d, *m, Some(*y)),
        _ => return None,
    };
    let number = |s: &str, max_len: usize| -> Option<u32> {
        (!s.is_empty() && s.len() <= max_len && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse().ok())
            .flatten()
    };
    let (day, month) = (number(d, 2)?, number(m, 2)?);
    match y {
        Some(y) => {
            let year = number(y, 4).filter(|_| y.len() == 2 || y.len() == 4)? as i32;
            let year = if year < 100 { year + 2000 } else { year };
            NaiveDate::from_ymd_opt(year, month, day)
        }
        None => (today.year()..=today.year() + 4)
            .filter_map(|year| NaiveDate::from_ymd_opt(year, month, day))
            .find(|date| *date >= today),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceType;

    /// Tuesday, 24 November 2026.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()
    }

    fn d(month: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(2026, month, day)
    }

    #[test]
    fn dates() {
        let next_year = NaiveDate::from_ymd_opt(2027, 1, 3);
        let cases: &[(&str, Option<NaiveDate>)] = &[
            ("heute", d(11, 24)),
            ("Heute", d(11, 24)),
            ("today", d(11, 24)),
            ("morgen", d(11, 25)),
            ("übermorgen", d(11, 26)),
            ("di", d(11, 24)),
            ("mi", d(11, 25)),
            ("fr", d(11, 27)),
            ("freitag", d(11, 27)),
            ("fri", d(11, 27)),
            ("mo", d(11, 30)),
            ("montag", d(11, 30)),
            ("so", d(11, 29)),
            ("su", d(11, 29)),
            ("28.11.", d(11, 28)),
            ("28.11", d(11, 28)),
            ("1.12.", d(12, 1)),
            ("3.1.", next_year),
            ("24.11.", d(11, 24)),
            ("28.11.2026", d(11, 28)),
            ("28.11.27", NaiveDate::from_ymd_opt(2027, 11, 28)),
            ("2026-12-24", d(12, 24)),
            ("29.02.", NaiveDate::from_ymd_opt(2028, 2, 29)),
            ("31.02.", None),
            ("wichtig", None),
            ("f", None),
            ("moin", None),
            ("1", None),
            ("1.2.3", None),
            ("12.13.", None),
        ];
        for (word, expected) in cases {
            assert_eq!(date(word, today()), *expected, "!{word}");
        }
    }

    fn url(target: &str) -> NewResource {
        NewResource {
            kind: classify(target).unwrap(),
            target: target.into(),
            label: None,
        }
    }

    #[test]
    fn examples_from_the_concept() {
        let cases: Vec<(&str, QuickEntry)> = vec![
            (
                "Release-Notes 3.4 schreiben",
                QuickEntry {
                    title: "Release-Notes 3.4 schreiben".into(),
                    ..Default::default()
                },
            ),
            (
                "Load-Test-Umgebung aufsetzen @Jonas",
                QuickEntry {
                    title: "Load-Test-Umgebung aufsetzen".into(),
                    delegate_to: Some("Jonas".into()),
                    ..Default::default()
                },
            ),
            (
                "Angebot prüfen !fr",
                QuickEntry {
                    title: "Angebot prüfen".into(),
                    date: d(11, 27),
                    ..Default::default()
                },
            ),
            (
                "Rechnung Q3 klären !28.11. @Buchhaltung",
                QuickEntry {
                    title: "Rechnung Q3 klären".into(),
                    date: d(11, 28),
                    delegate_to: Some("Buchhaltung".into()),
                    ..Default::default()
                },
            ),
            (
                "Review https://github.com/example/core/pull/42 !heute",
                QuickEntry {
                    title: "Review".into(),
                    date: d(11, 24),
                    resources: vec![url("https://github.com/example/core/pull/42")],
                    ..Default::default()
                },
            ),
            (
                "https://github.com/example/core/pull/42",
                QuickEntry {
                    title: "PR #42".into(),
                    resources: vec![url("https://github.com/example/core/pull/42")],
                    ..Default::default()
                },
            ),
            (
                "Spec lesen ~/projekte/core/docs/spec.md und https://jira.example.com/browse/CORE-418",
                QuickEntry {
                    title: "Spec lesen und".into(),
                    resources: vec![
                        url("~/projekte/core/docs/spec.md"),
                        url("https://jira.example.com/browse/CORE-418"),
                    ],
                    ..Default::default()
                },
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(parse(input, today()), expected, "{input}");
        }
    }

    #[test]
    fn words_that_only_look_like_shortcuts_stay() {
        let e = parse("Hilfe! Was ist !wichtig an @ und a@b.de?", today());
        assert_eq!(e.title, "Hilfe! Was ist !wichtig an @ und a@b.de?");
        assert_eq!(
            e,
            QuickEntry {
                title: e.title.clone(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn first_person_and_date_count_the_rest_stays() {
        let e = parse("Abstimmen mit @Frank und @Jonas !mo !fr", today());
        assert_eq!(e.delegate_to.as_deref(), Some("Frank"));
        assert_eq!(e.date, d(11, 30));
        assert_eq!(e.title, "Abstimmen mit und @Jonas !fr");
    }

    #[test]
    fn punctuation_around_shortcuts() {
        let e = parse("Frag @Jonas, siehe (https://x.example.com/a).", today());
        assert_eq!(e.delegate_to.as_deref(), Some("Jonas"));
        assert_eq!(e.title, "Frag siehe");
        assert_eq!(e.resources[0].target, "https://x.example.com/a");
        assert_eq!(
            parse("an @frank.mueller.", today()).delegate_to.as_deref(),
            Some("frank.mueller")
        );
    }

    #[test]
    fn mail_and_file_targets() {
        let e = parse("~/Mail/rechnung-0931.eml", today());
        assert_eq!(e.title, "Mail: rechnung-0931");
        assert_eq!(e.resources[0].kind, ResourceType::Mail);
        let e = parse("Logs ansehen /var/log/nginx/", today());
        assert_eq!(e.title, "Logs ansehen");
        assert_eq!(e.resources[0].kind, ResourceType::File);
    }

    #[test]
    fn target_follows_the_person() {
        let e = parse("x @Jonas", today());
        assert_eq!(e.target(Target::Now), Target::Delegated("Jonas".into()));
        assert_eq!(parse("x", today()).target(Target::Focus), Target::Focus);
        assert!(parse("   ", today()).is_empty());
        assert!(
            parse("@Jonas", today()).is_empty(),
            "a person alone is nothing to save"
        );
    }

    #[test]
    fn into_new_item_carries_everything() {
        let item = parse(
            "Review https://github.com/example/core/pull/42 !fr",
            today(),
        )
        .into_new_item();
        assert_eq!(item.title, "Review");
        assert_eq!(item.date, d(11, 27));
        assert_eq!(item.resources.len(), 1);
    }
}
