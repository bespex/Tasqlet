//! The few settings that are no data (`~/.config/tasqlet/settings.json`, plan 2.6 in a slim form):
//! the language, where the popover appears when the desktop gives no icon position and the global
//! key that shows or hides it (X11), and the appearance: colour theme and style from `@bespex/themes`.
//! A missing file means the defaults; a missing key does too.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::storage::atomic::write_atomic;

/// File name in the config folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// The language of the interface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// After the system language.
    #[default]
    Auto,
    De,
    En,
}

/// Where the popover appears without an icon position (Wayland, or `--toggle`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PopoverPosition {
    /// The top right corner of the screen.
    #[default]
    Corner,
    /// The middle of the screen.
    Center,
}

/// How Tasqlet opens, by tray click, global key and `--toggle` alike (D41, kit PLAN K9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowForm {
    /// At the tray icon, hidden when another window gets the focus unless pinned.
    #[default]
    Popover,
    /// Full height at an edge of the screen, stays open until hidden.
    Sidebar,
}

/// The edge of the sidebar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarSide {
    /// The half of the screen with the tray icon, as last seen on a tray click; else right.
    #[default]
    Auto,
    Left,
    Right,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub popover_position: PopoverPosition,
    pub window_form: WindowForm,
    pub sidebar_side: SidebarSide,
    /// Global key that shows or hides the popover, as `Ctrl+Alt+KeyT`; empty means none (D36).
    pub toggle_key: String,
    /// Colour theme and style, `{"theme": "fjord", "style": "calm"}` (plan 2.6). Kept as it comes:
    /// what the values mean is the business of `@bespex/themes` in the interface, which also
    /// replaces unknown ones. `None` until the user chooses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<serde_json::Value>,
}

impl Settings {
    /// Read the file; a missing file means the defaults.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_or_key_means_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
        std::fs::write(&path, r#"{"language":"de"}"#).unwrap();
        let s = Settings::load(&path).unwrap();
        assert_eq!(s.language, Language::De);
        assert_eq!(s.popover_position, PopoverPosition::Corner);
        assert_eq!(s.toggle_key, "");
        assert_eq!(s.window_form, WindowForm::Popover);
        assert_eq!(s.sidebar_side, SidebarSide::Auto);
        assert_eq!(s.appearance, None);
        // Keys from a newer version are ignored, not an error.
        std::fs::write(&path, r#"{"theme":"x","popover_position":"center"}"#).unwrap();
        assert_eq!(
            Settings::load(&path).unwrap().popover_position,
            PopoverPosition::Center
        );
    }

    #[test]
    fn saving_and_loading_agree_and_a_broken_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join(SETTINGS_FILE);
        let s = Settings {
            language: Language::En,
            popover_position: PopoverPosition::Center,
            window_form: WindowForm::Sidebar,
            sidebar_side: SidebarSide::Left,
            toggle_key: "Ctrl+Alt+KeyT".into(),
            appearance: Some(serde_json::json!({"theme": "fjord", "style": {"chrome": "raised"}})),
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), s);
        std::fs::write(&path, "{ nonsense").unwrap();
        assert!(Settings::load(&path).is_err());
        std::fs::write(&path, r#"{"language":"klingon"}"#).unwrap();
        assert!(Settings::load(&path).is_err());
    }
}
