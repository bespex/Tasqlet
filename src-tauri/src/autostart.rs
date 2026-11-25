//! Autostart at login: a `.desktop` file in the XDG autostart folder (works on every desktop that
//! follows the freedesktop autostart specification). With `TASQLET_CONFIG_DIR` set (tests,
//! development) the folder lies inside it, so a test never touches the real autostart.

use std::path::{Path, PathBuf};

const FILE: &str = "tasqlet.desktop";

/// The autostart folder.
fn folder() -> Option<PathBuf> {
    match tasqlet_core::jira::config_dir() {
        Some(dir) if std::env::var_os(tasqlet_core::jira::CONFIG_DIR_ENV).is_some() => {
            Some(dir.join("autostart"))
        }
        _ => dirs::config_dir().map(|dir| dir.join("autostart")),
    }
}

/// An `Exec` argument as the desktop entry specification wants it: quoted if it holds a space or a
/// character with a meaning there.
fn quote(path: &str) -> String {
    if !path.chars().any(|c| " \t\n\"'\\<>~|&;$*?#()`".contains(c)) {
        return path.to_string();
    }
    let mut out = String::from("\"");
    for c in path.chars() {
        if "\"`$\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// The content of the file for the program at `exe`.
fn entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Tasqlet\nComment=Focus and notes in the tray\nExec={}\nIcon=tasqlet\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        quote(&exe.to_string_lossy())
    )
}

/// Whether Tasqlet starts at login.
pub fn enabled() -> bool {
    folder().is_some_and(|dir| dir.join(FILE).is_file())
}

/// Switch autostart on or off.
pub fn set(on: bool) -> Result<(), String> {
    let dir = folder().ok_or("no config folder")?;
    let path = dir.join(FILE);
    if !on {
        return match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(&path, entry(&exe)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_is_quoted_only_when_needed() {
        assert_eq!(quote("/usr/bin/tasqlet"), "/usr/bin/tasqlet");
        assert_eq!(quote("/opt/my apps/tasqlet"), "\"/opt/my apps/tasqlet\"");
        assert_eq!(quote("/a/b\"c$"), "\"/a/b\\\"c\\$\"");
    }

    #[test]
    fn the_entry_is_a_valid_desktop_file() {
        let text = entry(Path::new("/usr/bin/tasqlet"));
        assert!(text.starts_with("[Desktop Entry]\nType=Application\n"));
        assert!(text.contains("\nExec=/usr/bin/tasqlet\n"));
        assert!(text.ends_with('\n'));
    }
}
