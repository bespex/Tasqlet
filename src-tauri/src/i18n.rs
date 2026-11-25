//! Texts the shell shows itself (tray menu, tooltip), in German or English after the system
//! language. The interface in the webview has its own i18n (plan step 1.5).

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    De,
    En,
}

static FORCED: OnceLock<Lang> = OnceLock::new();

/// Use this language whatever the system says (the setting). Only counts before the first text is
/// asked for, so it is called at startup; a change takes effect at the next start.
pub fn force(lang: Lang) {
    let _ = FORCED.set(lang);
}

/// The language: the one set in the settings, else from `LC_ALL`, `LC_MESSAGES` or `LANG` (German if
/// it starts with `de`).
pub fn lang() -> Lang {
    static LANG: OnceLock<Lang> = OnceLock::new();
    *LANG.get_or_init(|| {
        if let Some(forced) = FORCED.get() {
            return *forced;
        }
        let var = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty())
            .unwrap_or_default();
        if var.starts_with("de") {
            Lang::De
        } else {
            Lang::En
        }
    })
}

/// The text for `key`.
pub fn t(key: Key) -> &'static str {
    text(lang(), key)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Open,
    Capture,
    Pause,
    Resume,
    Finish,
    Quit,
    Idle,
    Ready,
    After,
    BreakThen,
    ClickHint,
    OpenAll,
}

fn text(lang: Lang, key: Key) -> &'static str {
    use Key::*;
    match (lang, key) {
        (Lang::De, Open) => "Öffnen",
        (Lang::En, Open) => "Open",
        (Lang::De, Capture) => "Schnell erfassen …",
        (Lang::En, Capture) => "Quick capture …",
        (Lang::De, Pause) => "Pause machen",
        (Lang::En, Pause) => "Take a break",
        (Lang::De, Resume) => "Weiter",
        (Lang::En, Resume) => "Resume",
        (Lang::De, Finish) => "Aktuelle Aufgabe erledigt",
        (Lang::En, Finish) => "Current task done",
        (Lang::De, Quit) => "Beenden",
        (Lang::En, Quit) => "Quit",
        (Lang::De, Idle) => "Nichts Dringendes",
        (Lang::En, Idle) => "Nothing urgent",
        (Lang::De, Ready) => "Bereit",
        (Lang::En, Ready) => "Ready",
        (Lang::De, After) => "Danach",
        (Lang::En, After) => "Next",
        (Lang::De, BreakThen) => "Pause. Danach weiter mit",
        (Lang::En, BreakThen) => "Break. Then back to",
        (Lang::De, OpenAll) => "Alle Arbeitsmittel öffnen",
        (Lang::En, OpenAll) => "Open all resources",
        (Lang::De, ClickHint) => "Klick öffnet Tasqlet",
        (Lang::En, ClickHint) => "Click to open Tasqlet",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_languages() {
        use Key::*;
        for key in [
            Open, Capture, Pause, Resume, Finish, Quit, Idle, Ready, After, BreakThen, ClickHint,
            OpenAll,
        ] {
            assert!(!text(Lang::De, key).is_empty());
            assert!(!text(Lang::En, key).is_empty());
        }
    }
}
