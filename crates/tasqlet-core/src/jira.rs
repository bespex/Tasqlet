//! Jira projects learned from links (`docs/KONZEPT.md` 5, D26).
//!
//! A link like `https://jira.firma.de/browse/NMS-19465` teaches the project: `NMS` belongs to
//! `https://jira.firma.de`. From then on a bare key `NMS-19470` can become a link. Learning is per
//! project, so `UTF-8` or `SHA-256` never turn into links, and several Jira instances can coexist.
//! The projects are kept in `~/.config/tasqlet/jira.json` and can be forgotten one by one.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::resources::is_issue_key;
use crate::storage::atomic::write_atomic;

/// File name in the config folder.
pub const JIRA_FILE: &str = "jira.json";
/// Environment variable that overrides the config folder (development, tests).
pub const CONFIG_DIR_ENV: &str = "TASQLET_CONFIG_DIR";

/// The config folder: `$TASQLET_CONFIG_DIR`, else `tasqlet` in the XDG config folder.
pub fn config_dir() -> Option<PathBuf> {
    match std::env::var_os(CONFIG_DIR_ENV) {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => dirs::config_dir().map(|dir| dir.join("tasqlet")),
    }
}

/// Learned projects: project key → base URL of its Jira (without `/browse/…`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JiraProjects {
    #[serde(default)]
    projects: BTreeMap<String, String>,
}

impl JiraProjects {
    /// Read the file; a missing file means nothing learned yet.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    /// Write the file atomically, creating the folder if needed.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        bytes.push(b'\n');
        write_atomic(path, &bytes, false)
    }

    /// Learn the project of a Jira link. Returns the project if something new was learned or its
    /// base URL changed; `None` for other links and for what was known already.
    pub fn learn(&mut self, url: &str) -> Option<String> {
        let (base, key) = split_browse_link(url)?;
        let project = key.split('-').next()?.to_string();
        if self.projects.get(&project).map(String::as_str) == Some(base) {
            return None;
        }
        self.projects.insert(project.clone(), base.to_string());
        Some(project)
    }

    /// The link for a key of a learned project, e.g. `NMS-19470` →
    /// `https://jira.firma.de/browse/NMS-19470`.
    pub fn link(&self, key: &str) -> Option<String> {
        if !is_issue_key(key) {
            return None;
        }
        let project = key.split('-').next()?;
        self.projects
            .get(project)
            .map(|base| format!("{base}/browse/{key}"))
    }

    /// Forget a project. Returns whether it was known.
    pub fn forget(&mut self, project: &str) -> bool {
        self.projects.remove(project).is_some()
    }

    /// Learned projects with their base URL, sorted by project.
    pub fn projects(&self) -> impl Iterator<Item = (&str, &str)> {
        self.projects.iter().map(|(p, b)| (p.as_str(), b.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }
}

/// `https://host/ctx/browse/KEY-1?x` → (`https://host/ctx`, `KEY-1`).
fn split_browse_link(url: &str) -> Option<(&str, &str)> {
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return None;
    }
    let at = url.find("/browse/")?;
    let base = &url[..at];
    let rest = &url[at + "/browse/".len()..];
    let key = &rest[..rest.find(['/', '?', '#']).unwrap_or(rest.len())];
    (is_issue_key(key) && base.len() > "https://".len()).then_some((base, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learns_the_project_and_links_its_keys() {
        let mut jira = JiraProjects::default();
        assert_eq!(
            jira.learn("https://jira.firma.de/browse/NMS-19465"),
            Some("NMS".into())
        );
        assert_eq!(
            jira.link("NMS-19470").as_deref(),
            Some("https://jira.firma.de/browse/NMS-19470")
        );
        assert_eq!(
            jira.learn("https://jira.firma.de/browse/NMS-1?focus=2"),
            None,
            "known"
        );
        assert_eq!(jira.link("CORE-1"), None, "other project");
        assert_eq!(jira.link("UTF-8"), None);
        assert_eq!(jira.link("nms-1"), None, "not a key");
    }

    #[test]
    fn several_instances_and_context_paths() {
        let mut jira = JiraProjects::default();
        jira.learn("https://jira.firma.de/browse/NMS-1");
        jira.learn("https://example.com/jira/browse/CORE-418#comment");
        assert_eq!(
            jira.link("CORE-9").as_deref(),
            Some("https://example.com/jira/browse/CORE-9")
        );
        assert_eq!(
            jira.projects().collect::<Vec<_>>(),
            [
                ("CORE", "https://example.com/jira"),
                ("NMS", "https://jira.firma.de")
            ]
        );
        // A project that moved gets the new address.
        assert_eq!(
            jira.learn("https://new.firma.de/browse/NMS-2"),
            Some("NMS".into())
        );
        assert!(jira.forget("NMS"));
        assert!(!jira.forget("NMS"));
        assert_eq!(jira.link("NMS-2"), None);
    }

    #[test]
    fn ignores_what_is_no_jira_link() {
        let mut jira = JiraProjects::default();
        for url in [
            "https://github.com/example/core/pull/42",
            "https://x.de/browse/kein-key",
            "~/browse/NMS-1",
            "https:///browse/NMS-1",
        ] {
            assert_eq!(jira.learn(url), None, "{url}");
        }
        assert!(jira.is_empty());
    }

    #[test]
    fn file_round_trip_and_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join(JIRA_FILE);
        assert!(JiraProjects::load(&path).unwrap().is_empty());
        let mut jira = JiraProjects::default();
        jira.learn("https://jira.firma.de/browse/NMS-1");
        jira.save(&path).unwrap();
        assert_eq!(JiraProjects::load(&path).unwrap(), jira);
        std::fs::write(&path, b"{ kaputt").unwrap();
        assert!(JiraProjects::load(&path).is_err());
    }
}
