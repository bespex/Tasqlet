//! Noticing changes to the data files made by others (CLI, editor, sync tool).
//!
//! Own writes are recognised by content, not by timing: the storage remembers what it last wrote
//! or read for each file, and a change event whose file still holds exactly that is ignored.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use notify::{EventKind, RecursiveMode, Watcher as _};

use super::{Storage, is_data_file};

/// Keeps watching while it lives.
pub struct Watcher {
    _inner: notify::RecommendedWatcher,
}

impl Storage {
    /// Call `on_change` with the path of a data file whenever someone else changed it. The app
    /// then loads again. The callback runs on the watcher's thread; one external write can be
    /// reported more than once if it arrives in several steps.
    pub fn watch(&self, on_change: impl Fn(PathBuf) + Send + 'static) -> notify::Result<Watcher> {
        let storage = self.clone();
        let dir = self.dir().to_path_buf();
        // Last content reported per file, so repeated events for one write are reported once.
        let reported: Mutex<HashMap<PathBuf, Option<u64>>> = Mutex::new(HashMap::new());
        let mut inner = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else { return };
            if !matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                return;
            }
            for path in event.paths {
                if !is_data_file(&dir, &path) {
                    continue;
                }
                let Ok(foreign) = storage.foreign_content(&path) else {
                    continue;
                };
                let mut reported = reported.lock().unwrap_or_else(|e| e.into_inner());
                let Some(fingerprint) = foreign else {
                    // Own content again: the same foreign content later is news again.
                    reported.remove(&path);
                    continue;
                };
                if reported.get(&path) == Some(&fingerprint) {
                    continue;
                }
                reported.insert(path.clone(), fingerprint);
                drop(reported);
                on_change(path);
            }
        })?;
        inner.watch(self.dir(), RecursiveMode::Recursive)?;
        Ok(Watcher { _inner: inner })
    }
}
