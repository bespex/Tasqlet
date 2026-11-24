//! Storage of the items on disk (`docs/KONZEPT.md` 2.2 and 2.5).
//!
//! ```text
//! <data dir>/items.json          everything that is not done, plus the break flag
//! <data dir>/archive/<year>.json done items, by the year they were finished (local time)
//! ```
//!
//! In memory the [`Store`] holds all items, done ones included, so undo and search need no extra
//! loading. Every file is replaced atomically and keeps its previous version as `.bak`.
//!
//! When an item moves from one file to another (finished, or reopened from the archive), the save
//! runs in two phases: first every file is written with its new items while still keeping the
//! ones that leave it, then the files that lose items are written again. A crash in between can
//! only leave an item in two files, never in none; loading keeps the copy changed last.
//!
//! The storage remembers what it last wrote or read for each file, so the watcher can tell its own
//! writes from changes made by others ([`Storage::watch`]).

mod atomic;
mod migrate;
mod saver;
mod watch;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION;
use crate::model::{Id, Item, State};
use crate::store::Store;

pub use atomic::backup_path;
pub use saver::{SAVE_DELAY, Saver};
pub use watch::Watcher;

use migrate::MigrationError;

/// File name of the main data file.
pub const ITEMS_FILE: &str = "items.json";
/// Folder of the yearly archive files.
pub const ARCHIVE_DIR: &str = "archive";
/// Environment variable that overrides the data folder (development, tests, portable use).
pub const DATA_DIR_ENV: &str = "TASQLET_DATA_DIR";

/// The data folder: `$TASQLET_DATA_DIR` if set, else `tasqlet` in the XDG data folder
/// (`~/.local/share/tasqlet`).
pub fn default_dir() -> Option<PathBuf> {
    match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => dirs::data_dir().map(|dir| dir.join("tasqlet")),
    }
}

/// Why the data could not be loaded. Nothing has been changed on disk.
#[derive(Debug)]
pub enum LoadError {
    /// The file could not be read.
    Io { path: PathBuf, source: io::Error },
    /// The file is not valid; `backup` is the previous version, if there is one to offer.
    Corrupt {
        path: PathBuf,
        message: String,
        backup: Option<PathBuf>,
    },
    /// The file was written by a newer Tasqlet. It must not be overwritten.
    TooNew { path: PathBuf, schema: u64 },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            LoadError::Corrupt {
                path,
                message,
                backup,
            } => {
                write!(f, "{} is damaged: {message}", path.display())?;
                if let Some(bak) = backup {
                    write!(f, " (backup available: {})", bak.display())?;
                }
                Ok(())
            }
            LoadError::TooNew { path, schema } => write!(
                f,
                "{} has schema {schema}, this version reads up to {SCHEMA_VERSION}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Result of [`Storage::load`].
#[derive(Debug)]
pub struct Loaded {
    pub store: Store,
    /// Something broke an invariant (e.g. after editing by hand) and was repaired.
    pub repaired: bool,
    /// A file had an older `schema` and was migrated. It is written in the new format with the
    /// next save.
    pub migrated: bool,
}

/// Access to the data folder. Cheap to clone; clones share what they know about the files.
#[derive(Debug, Clone)]
pub struct Storage {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    dir: PathBuf,
    /// What each file contained when this storage last read or wrote it. The lock also keeps
    /// saves from running at the same time.
    known: Mutex<HashMap<PathBuf, Known>>,
}

#[derive(Debug, Clone)]
struct Known {
    hash: u64,
    ids: HashSet<Id>,
}

#[derive(Deserialize)]
struct ItemsFile {
    #[serde(default)]
    on_break: bool,
    #[serde(default)]
    items: Vec<Item>,
}

#[derive(Serialize)]
struct ItemsFileOut<'a> {
    schema: u32,
    on_break: bool,
    items: Vec<&'a Item>,
}

#[derive(Serialize)]
struct ArchiveFileOut<'a> {
    schema: u32,
    items: Vec<&'a Item>,
}

impl Storage {
    /// Use the given folder, creating it and its archive folder if needed.
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(dir.join(ARCHIVE_DIR))?;
        Ok(Self {
            inner: Arc::new(Inner {
                dir,
                known: Mutex::new(HashMap::new()),
            }),
        })
    }

    /// The data folder.
    pub fn dir(&self) -> &Path {
        &self.inner.dir
    }

    /// Path of `items.json`.
    pub fn items_path(&self) -> PathBuf {
        self.inner.dir.join(ITEMS_FILE)
    }

    /// Path of the archive file of a year.
    pub fn archive_path(&self, year: i32) -> PathBuf {
        self.inner
            .dir
            .join(ARCHIVE_DIR)
            .join(format!("{year}.json"))
    }

    /// Read `items.json` and all archive files. A missing `items.json` means a first start and
    /// gives an empty store.
    pub fn load(&self) -> Result<Loaded, LoadError> {
        let mut known = self.lock();
        let mut found = HashMap::new();
        let mut migrated = false;
        let mut on_break = false;
        let mut items: Vec<Item> = Vec::new();

        let items_path = self.items_path();
        if let Some(bytes) = read_optional(&items_path)? {
            let (value, m) = parse(&items_path, &bytes)?;
            let file: ItemsFile = from_value(&items_path, value)?;
            migrated |= m;
            on_break = file.on_break;
            found.insert(items_path, known_of(&bytes, &file.items));
            items.extend(file.items);
        }
        for path in self.archive_files().map_err(|source| LoadError::Io {
            path: self.inner.dir.join(ARCHIVE_DIR),
            source,
        })? {
            let Some(bytes) = read_optional(&path)? else {
                continue;
            };
            let (value, m) = parse(&path, &bytes)?;
            let file: ItemsFile = from_value(&path, value)?;
            migrated |= m;
            found.insert(path, known_of(&bytes, &file.items));
            items.extend(file.items);
        }

        let (store, repaired) = Store::repaired(on_break, newest_per_id(items));
        *known = found;
        Ok(Loaded {
            store,
            repaired,
            migrated,
        })
    }

    /// Write the store. Files whose content would not change are left alone.
    pub fn save(&self, store: &Store) -> io::Result<()> {
        let mut known = self.lock();
        let targets = self.targets(store, &known);
        let known_ids: HashMap<&Path, &HashSet<Id>> =
            known.iter().map(|(p, k)| (p.as_path(), &k.ids)).collect();
        let phases = plan(&targets, &known_ids);
        drop(known_ids);
        for phase in phases {
            for (path, items) in phase {
                self.write_file(&mut known, &path, store, items)?;
            }
        }
        Ok(())
    }

    /// Put the backup of a damaged file in its place. The damaged file is kept as `.broken`.
    /// Load again afterwards.
    pub fn restore_backup(&self, path: &Path) -> io::Result<()> {
        let bak = backup_path(path);
        let bytes = fs::read(&bak)?;
        if path.exists() {
            fs::rename(path, atomic::broken_path(path))?;
        }
        atomic::write_atomic(path, &bytes, false)?;
        self.lock().remove(path);
        Ok(())
    }

    /// Read `path` and compare it with what this storage last wrote or read there. Returns `None`
    /// if it is the own content, else a fingerprint of the foreign content (`Some(None)` if the
    /// file was removed). Runs under the lock, so a save in progress finishes first and its
    /// intermediate content is never mistaken for a foreign one.
    pub(crate) fn foreign_content(&self, path: &Path) -> io::Result<Option<Option<u64>>> {
        let known = self.lock();
        let current = match fs::read(path) {
            Ok(bytes) => Some(hash(&bytes)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        let own = known.get(path).map(|k| k.hash);
        Ok((current != own).then_some(current))
    }

    /// Lock what is known about the files. Every user of the lock leaves the map consistent, so
    /// a panic elsewhere does not make it unusable.
    fn lock(&self) -> MutexGuard<'_, HashMap<PathBuf, Known>> {
        self.inner
            .known
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn archive_files(&self) -> io::Result<Vec<PathBuf>> {
        let dir = self.inner.dir.join(ARCHIVE_DIR);
        let mut files = Vec::new();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(files),
            Err(e) => return Err(e),
        };
        for entry in entries {
            let path = entry?.path();
            if is_archive_name(&path) {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    /// Which items belong in which file. Archive files known from before that end up empty are
    /// included, so their items are removed.
    fn targets<'a>(
        &self,
        store: &'a Store,
        known: &HashMap<PathBuf, Known>,
    ) -> BTreeMap<PathBuf, Vec<&'a Item>> {
        let mut targets: BTreeMap<PathBuf, Vec<&Item>> = BTreeMap::new();
        targets.insert(self.items_path(), Vec::new());
        for path in known.keys() {
            targets.entry(path.clone()).or_default();
        }
        for it in store.items() {
            let path = if it.state == State::Done {
                let finished = it.done_at.unwrap_or(it.updated_at);
                self.archive_path(finished.with_timezone(&Local).year())
            } else {
                self.items_path()
            };
            targets.entry(path).or_default().push(it);
        }
        targets
    }

    fn write_file(
        &self,
        known: &mut HashMap<PathBuf, Known>,
        path: &Path,
        store: &Store,
        mut items: Vec<&Item>,
    ) -> io::Result<()> {
        items.sort_by_key(|it| file_order(it));
        let is_items_file = path == self.items_path();
        if !is_items_file && items.is_empty() {
            match fs::remove_file(path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
                _ => {}
            }
            known.remove(path);
            return Ok(());
        }
        let mut bytes = if is_items_file {
            serde_json::to_vec_pretty(&ItemsFileOut {
                schema: SCHEMA_VERSION,
                on_break: store.on_break(),
                items: items.clone(),
            })
        } else {
            serde_json::to_vec_pretty(&ArchiveFileOut {
                schema: SCHEMA_VERSION,
                items: items.clone(),
            })
        }
        .map_err(io::Error::other)?;
        bytes.push(b'\n');
        let new = Known {
            hash: hash(&bytes),
            ids: items.iter().map(|it| it.id).collect(),
        };
        if known.get(path).is_some_and(|k| k.hash == new.hash) {
            return Ok(());
        }
        atomic::write_atomic(path, &bytes, worth_backing_up(path, known.get(path)))?;
        known.insert(path.to_path_buf(), new);
        Ok(())
    }
}

/// The writes of one save, in phases. Phase one writes every file with its final items plus the
/// items that move away from it to another file; phase two, only needed if items move, writes
/// those files again with their final items. So every item is in at least one file at any time.
fn plan<'a>(
    targets: &BTreeMap<PathBuf, Vec<&'a Item>>,
    known_ids: &HashMap<&Path, &HashSet<Id>>,
) -> Vec<Vec<(PathBuf, Vec<&'a Item>)>> {
    let placed: HashMap<Id, &Path> = targets
        .iter()
        .flat_map(|(path, items)| items.iter().map(move |it| (it.id, path.as_path())))
        .collect();
    let by_id: HashMap<Id, &'a Item> = targets.values().flatten().map(|it| (it.id, *it)).collect();
    let mut first = Vec::new();
    let mut second = Vec::new();
    for (path, items) in targets {
        let leaving: Vec<&Item> = known_ids
            .get(path.as_path())
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter(|id| placed.get(id).is_some_and(|p| *p != path.as_path()))
            .map(|id| by_id[id])
            .collect();
        if leaving.is_empty() {
            first.push((path.clone(), items.clone()));
        } else {
            let mut keep = items.clone();
            keep.extend(leaving);
            first.push((path.clone(), keep));
            second.push((path.clone(), items.clone()));
        }
    }
    if second.is_empty() {
        vec![first]
    } else {
        vec![first, second]
    }
}

/// Only keep the current file as backup if it is something worth keeping: what this storage wrote
/// or read itself, or at least valid JSON. A damaged file must not replace a good backup.
fn worth_backing_up(path: &Path, known: Option<&Known>) -> bool {
    let Ok(current) = fs::read(path) else {
        return false;
    };
    known.is_some_and(|k| k.hash == hash(&current))
        || serde_json::from_slice::<serde_json::Value>(&current).is_ok()
}

/// Stable order of items in a file, so that saves produce small diffs.
fn file_order(it: &Item) -> (u8, u32, chrono::DateTime<chrono::Utc>, Id) {
    let rank = match it.state {
        State::Focus => 0,
        State::Intermezzo => 1,
        State::Idea => 2,
        State::Delegated => 3,
        State::Done => 4,
    };
    let when = if it.state == State::Done {
        it.done_at.unwrap_or(it.updated_at)
    } else {
        it.created_at
    };
    (rank, it.order, when, it.id)
}

/// If an item is in more than one file (crash between the two phases of a save, or an edit by
/// hand), keep the copy changed last.
fn newest_per_id(items: Vec<Item>) -> Vec<Item> {
    let mut index: HashMap<Id, usize> = HashMap::new();
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    for it in items {
        match index.get(&it.id) {
            Some(&i) => {
                if it.updated_at > out[i].updated_at {
                    out[i] = it;
                }
            }
            None => {
                index.insert(it.id, out.len());
                out.push(it);
            }
        }
    }
    out
}

fn is_archive_name(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "json")
        && path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `path` is one of the data files in `dir` (not a temporary or backup file).
pub(crate) fn is_data_file(dir: &Path, path: &Path) -> bool {
    path == dir.join(ITEMS_FILE)
        || (path.parent() == Some(dir.join(ARCHIVE_DIR).as_path()) && is_archive_name(path))
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, LoadError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(LoadError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn corrupt(path: &Path, message: String) -> LoadError {
    let bak = backup_path(path);
    LoadError::Corrupt {
        path: path.to_path_buf(),
        message,
        backup: bak.exists().then_some(bak),
    }
}

fn parse(path: &Path, bytes: &[u8]) -> Result<(serde_json::Value, bool), LoadError> {
    let value = serde_json::from_slice(bytes).map_err(|e| corrupt(path, e.to_string()))?;
    migrate::migrate(value).map_err(|e| match e {
        MigrationError::TooNew(schema) => LoadError::TooNew {
            path: path.to_path_buf(),
            schema,
        },
        MigrationError::Invalid(message) => corrupt(path, message),
    })
}

fn from_value<T: serde::de::DeserializeOwned>(
    path: &Path,
    value: serde_json::Value,
) -> Result<T, LoadError> {
    serde_json::from_value(value).map_err(|e| corrupt(path, e.to_string()))
}

fn hash(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

fn known_of(bytes: &[u8], items: &[Item]) -> Known {
    Known {
        hash: hash(bytes),
        ids: items.iter().map(|it| it.id).collect(),
    }
}

#[cfg(test)]
mod tests;
