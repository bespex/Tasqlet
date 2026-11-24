//! Work resources: recognising links, files and mails, their smart labels and their rank
//! (`docs/KONZEPT.md` 5).

use std::cmp::Reverse;

use crate::model::{Resource, ResourceType};

/// What kind of resource `raw` is, if it is one at all: an `http(s)` link, a mail (`mailto:`,
/// `mid:`, `.eml` file) or a file or folder (`/…`, `~/…`, `file://…`). Text with spaces is none.
pub fn classify(raw: &str) -> Option<ResourceType> {
    let raw = raw.trim();
    if raw.is_empty() || raw.chars().any(char::is_whitespace) {
        return None;
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return host(raw).is_some().then_some(ResourceType::Url);
    }
    if lower.starts_with("mailto:") || lower.starts_with("mid:") || lower.ends_with(".eml") {
        return Some(ResourceType::Mail);
    }
    let is_path = lower.starts_with("file://")
        || (raw.starts_with('/') && raw.len() > 1)
        || (raw.starts_with("~/") && raw.len() > 2);
    is_path.then_some(ResourceType::File)
}

/// The label shown for a target without an own label:
///
/// | target | label |
/// |---|---|
/// | `https://jira…/browse/CORE-418` | `CORE-418` |
/// | `https://github.com/o/repo/pull/42` | `PR #42` |
/// | `https://github.com/o/repo/issues/12` | `repo#12` |
/// | other link | host without `www.` |
/// | `~/Mail/rechnung.eml`, `mailto:a@b.de`, `mid:x` | `Mail: rechnung`, `Mail: a@b.de`, `Mail: x` |
/// | file or folder | its name |
pub fn smart_label(target: &str) -> String {
    let target = target.trim();
    match classify(target) {
        Some(ResourceType::Url) => url_label(target),
        Some(ResourceType::Mail) => mail_label(target),
        Some(ResourceType::File) => file_name(target).to_string(),
        None => target.to_string(),
    }
}

impl Resource {
    /// The own label, or the smart label if there is none.
    pub fn shown_label(&self) -> String {
        match &self.label {
            Some(label) if !label.trim().is_empty() => label.clone(),
            _ => smart_label(&self.target),
        }
    }
}

/// A resource in display order, with its fixed key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ranked<'a> {
    pub resource: &'a Resource,
    /// Position in the order of adding, from 0.
    pub index: usize,
    /// The digit for `Alt+n`: fixed to the resource (order of adding), 1 to 9, `None` beyond.
    pub digit: Option<u8>,
}

/// Resources by importance: last opened first (`used_at` descending), never opened ones after
/// them in the order of adding. The first three are shown at the work card (D6).
pub fn ranked(resources: &[Resource]) -> Vec<Ranked<'_>> {
    let mut list: Vec<Ranked<'_>> = resources
        .iter()
        .enumerate()
        .map(|(index, resource)| Ranked {
            resource,
            index,
            digit: u8::try_from(index + 1).ok().filter(|d| *d <= 9),
        })
        .collect();
    list.sort_by_key(|r| (Reverse(r.resource.used_at), r.index));
    list
}

/// Host of an `http(s)` link, without user info and port.
fn host(url: &str) -> Option<&str> {
    let rest = &url[url.find("://")? + 3..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = match host.rfind(':') {
        Some(i) if !host.ends_with(']') => &host[..i],
        _ => host,
    };
    (!host.is_empty()).then_some(host)
}

/// Path segments of a link, without query and fragment.
fn segments(url: &str) -> Vec<&str> {
    let Some(start) = url.find("://") else {
        return Vec::new();
    };
    let rest = &url[start + 3..];
    let rest = &rest[..rest.find(['?', '#']).unwrap_or(rest.len())];
    rest.split('/').skip(1).filter(|s| !s.is_empty()).collect()
}

fn url_label(url: &str) -> String {
    let segs = segments(url);
    if let Some(key) = segs
        .windows(2)
        .find(|w| w[0] == "browse" && is_issue_key(w[1]))
        .map(|w| w[1])
    {
        return key.to_string();
    }
    let host = host(url).unwrap_or(url);
    if (host.eq_ignore_ascii_case("github.com") || host.eq_ignore_ascii_case("www.github.com"))
        && let [_, repo, kind, number, ..] = segs.as_slice()
        && number.bytes().all(|b| b.is_ascii_digit())
    {
        match *kind {
            "pull" => return format!("PR #{number}"),
            "issues" => return format!("{repo}#{number}"),
            _ => {}
        }
    }
    host.strip_prefix("www.").unwrap_or(host).to_string()
}

/// `CORE-418`: a project key (capital letter, then capitals or digits), a dash, a number.
fn is_issue_key(s: &str) -> bool {
    let Some((project, number)) = s.split_once('-') else {
        return false;
    };
    let mut chars = project.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && !number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit())
}

fn mail_label(target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    let name = if lower.starts_with("mailto:") {
        let address = &target["mailto:".len()..];
        address.split('?').next().unwrap_or(address)
    } else if lower.starts_with("mid:") {
        &target["mid:".len()..]
    } else {
        let name = file_name(target);
        &name[..name.len() - ".eml".len().min(name.len())]
    };
    format!("Mail: {name}")
}

/// Last segment of a path, ignoring a trailing slash.
fn file_name(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    if name.is_empty() { path } else { name }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ulid::Ulid;

    #[test]
    fn smart_labels() {
        let cases = [
            ("https://jira.example.com/browse/CORE-418", "CORE-418"),
            (
                "https://jira.example.com/browse/CORE-418?focusedId=1#c",
                "CORE-418",
            ),
            ("https://github.com/example/core/pull/42", "PR #42"),
            ("https://github.com/example/core/pull/42/files", "PR #42"),
            ("https://github.com/example/worker/issues/212", "worker#212"),
            ("https://github.com/example/core", "github.com"),
            ("https://www.heise.de/news/x.html", "heise.de"),
            (
                "http://user@staging.example.com:8080/health",
                "staging.example.com",
            ),
            ("https://example.com/browse/kein-key", "example.com"),
            ("~/projekte/core/docs/spec.md", "spec.md"),
            ("~/projekte/core/gateway/", "gateway"),
            ("/etc/nginx/nginx.conf", "nginx.conf"),
            (
                "file:///home/me/Dokumente/arch-review.odp",
                "arch-review.odp",
            ),
            ("~/Mail/rechnung-0931.eml", "Mail: rechnung-0931"),
            (
                "mailto:ops@example.com?subject=Webhook",
                "Mail: ops@example.com",
            ),
            (
                "mid:abc123@mail.example.com",
                "Mail: abc123@mail.example.com",
            ),
        ];
        for (target, label) in cases {
            assert_eq!(smart_label(target), label, "{target}");
        }
    }

    #[test]
    fn classify_recognises_only_resources() {
        assert_eq!(classify("https://x.de"), Some(ResourceType::Url));
        assert_eq!(classify("HTTPS://X.DE/a"), Some(ResourceType::Url));
        assert_eq!(classify("~/a.eml"), Some(ResourceType::Mail));
        assert_eq!(classify("mailto:a@b.de"), Some(ResourceType::Mail));
        assert_eq!(classify("~/Dokumente"), Some(ResourceType::File));
        assert_eq!(classify("/tmp/x"), Some(ResourceType::File));
        for none in [
            "",
            "/",
            "~/",
            "https://",
            "Text",
            "und/oder",
            "a b",
            "ftp://x.de",
        ] {
            assert_eq!(classify(none), None, "{none:?}");
        }
    }

    fn res(target: &str, used_at: Option<u32>, label: Option<&str>) -> Resource {
        Resource {
            id: Ulid::generate(),
            kind: classify(target).unwrap(),
            target: target.into(),
            label: label.map(Into::into),
            used_at: used_at.map(|m| Utc.with_ymd_and_hms(2026, 11, 24, 9, m, 0).unwrap()),
        }
    }

    #[test]
    fn own_label_wins_over_smart_label() {
        assert_eq!(
            res("~/x.eml", None, Some("Mail: Ops")).shown_label(),
            "Mail: Ops"
        );
        assert_eq!(res("~/x.eml", None, Some("  ")).shown_label(), "Mail: x");
    }

    #[test]
    fn rank_by_last_use_then_order_with_fixed_digits() {
        let list = [
            res("https://jira.example.com/browse/CORE-418", None, None),
            res("~/spec.md", Some(5), None),
            res("https://github.com/example/core/pull/57", None, None),
            res("~/gateway/", Some(20), None),
        ];
        let ranked = ranked(&list);
        let shown: Vec<(String, Option<u8>)> = ranked
            .iter()
            .map(|r| (r.resource.shown_label(), r.digit))
            .collect();
        assert_eq!(
            shown,
            [
                ("gateway".into(), Some(4)),
                ("spec.md".into(), Some(2)),
                ("CORE-418".into(), Some(1)),
                ("PR #57".into(), Some(3)),
            ]
        );
    }

    #[test]
    fn digits_stop_at_nine() {
        let list: Vec<Resource> = (0..11)
            .map(|i| res(&format!("~/f{i}"), None, None))
            .collect();
        let digits: Vec<Option<u8>> = ranked(&list).iter().map(|r| r.digit).collect();
        assert_eq!(digits[8], Some(9));
        assert_eq!(digits[9], None);
        assert_eq!(digits[10], None);
    }
}
