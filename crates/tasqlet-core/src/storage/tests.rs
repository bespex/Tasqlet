use std::sync::mpsc;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use super::*;
use crate::model::{NewItem, Target};
use crate::store::List;

fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
}

fn add(store: &mut Store, title: &str, target: Target, when: DateTime<Utc>) -> Id {
    let mut new = NewItem::new(title);
    new.note = format!("Notiz zu {title}");
    new.checklist = vec!["Schritt".into()];
    let id = new.id;
    store.add(new, target, when).unwrap();
    id
}

/// Focus `a* b`, intermezzo `z`, idea `i`, delegated `d`, done `x` (2025) and `y` (2026).
fn sample() -> (Store, [Id; 7]) {
    let mut s = Store::new();
    let t = at(2026, 11, 20);
    let a = add(&mut s, "a", Target::Now, t);
    let b = add(&mut s, "b", Target::Focus, t);
    let z = add(&mut s, "z", Target::Intermezzo, t);
    let i = add(&mut s, "i", Target::Idea, t);
    let d = add(&mut s, "d", Target::Delegated("Jonas".into()), t);
    let x = add(&mut s, "x", Target::Idea, t);
    let y = add(&mut s, "y", Target::Idea, t);
    s.finish(x, at(2025, 6, 1)).unwrap();
    s.finish(y, at(2026, 11, 21)).unwrap();
    s.pause().unwrap();
    (s, [a, b, z, i, d, x, y])
}

fn sorted(store: &Store) -> Vec<Item> {
    let mut items = store.items().to_vec();
    items.sort_by_key(|it| it.id);
    items
}

fn open() -> (tempfile::TempDir, Storage) {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::open(dir.path()).unwrap();
    (dir, storage)
}

fn ids_in(path: &Path) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    v["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn first_start_gives_an_empty_store() {
    let (_dir, storage) = open();
    let loaded = storage.load().unwrap();
    assert!(loaded.store.items().is_empty());
    assert!(!loaded.repaired && !loaded.migrated);
}

#[test]
fn round_trip_keeps_everything() {
    let (_dir, storage) = open();
    let (store, _) = sample();
    storage.save(&store).unwrap();
    let loaded = Storage::open(storage.dir()).unwrap().load().unwrap();
    assert!(!loaded.repaired, "saved data needs no repair");
    assert_eq!(sorted(&loaded.store), sorted(&store));
    assert!(loaded.store.on_break());
    loaded.store.check_invariants().unwrap();
}

#[test]
fn done_items_go_to_the_archive_of_their_year() {
    let (_dir, storage) = open();
    let (store, _) = sample();
    storage.save(&store).unwrap();
    assert_eq!(ids_in(&storage.items_path()), ["a", "b", "z", "i", "d"]);
    assert_eq!(ids_in(&storage.archive_path(2025)), ["x"]);
    assert_eq!(ids_in(&storage.archive_path(2026)), ["y"]);
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(storage.items_path()).unwrap()).unwrap();
    assert_eq!(raw["schema"], 1);
    assert_eq!(raw["on_break"], true);
    assert_eq!(raw["items"][0]["state"], "focus");
    assert_eq!(raw["items"][0]["active"], true);
}

#[test]
fn reopening_an_archived_item_empties_its_year_file() {
    let (_dir, storage) = open();
    let (mut store, [.., x, _]) = sample();
    storage.save(&store).unwrap();
    store.send_to(x, List::Focus, at(2026, 11, 22)).unwrap();
    storage.save(&store).unwrap();
    assert!(!storage.archive_path(2025).exists());
    assert!(ids_in(&storage.items_path()).contains(&"x".to_string()));
    assert_eq!(sorted(&storage.load().unwrap().store), sorted(&store));
}

#[test]
fn unchanged_files_are_not_rewritten() {
    let (_dir, storage) = open();
    let (store, _) = sample();
    storage.save(&store).unwrap();
    let modified = |p: PathBuf| fs::metadata(p).unwrap().modified().unwrap();
    let before = modified(storage.archive_path(2025));
    std::thread::sleep(Duration::from_millis(20));
    storage.save(&store).unwrap();
    assert_eq!(modified(storage.archive_path(2025)), before);
    assert!(
        !backup_path(&storage.items_path()).exists(),
        "no second write, no backup"
    );
}

#[test]
fn moving_items_never_leaves_an_item_in_no_file() {
    let (_dir, storage) = open();
    let (mut store, [a, .., x, _]) = sample();
    storage.save(&store).unwrap();
    // One save that moves items in both directions.
    store.finish(a, at(2025, 7, 1)).unwrap();
    store.send_to(x, List::Idea, at(2026, 11, 22)).unwrap();
    let known = storage.lock().clone();
    let targets = storage.targets(&store, &known);
    let known_ids: HashMap<&Path, &HashSet<Id>> =
        known.iter().map(|(p, k)| (p.as_path(), &k.ids)).collect();
    let phases = plan(&targets, &known_ids);
    assert_eq!(phases.len(), 2);
    // Simulate the files after each single write; every item must be somewhere at all times.
    let mut files: HashMap<PathBuf, HashSet<Id>> = known
        .iter()
        .map(|(p, k)| (p.clone(), k.ids.clone()))
        .collect();
    let all: HashSet<Id> = store.items().iter().map(|it| it.id).collect();
    for phase in &phases {
        for (path, items) in phase {
            files.insert(path.clone(), items.iter().map(|it| it.id).collect());
            let present: HashSet<Id> = files.values().flatten().copied().collect();
            assert!(
                all.is_subset(&present),
                "an item went missing after writing {path:?}"
            );
        }
    }
    storage.save(&store).unwrap();
    assert_eq!(sorted(&storage.load().unwrap().store), sorted(&store));
}

#[test]
fn an_item_in_two_files_keeps_the_newer_copy() {
    let (_dir, storage) = open();
    let (store, [a, ..]) = sample();
    storage.save(&store).unwrap();
    // As if a crash happened between the phases: an older copy of `a` stayed in the archive.
    let mut old = store.get(a).unwrap().clone();
    old.title = "alt".into();
    old.updated_at = at(2020, 1, 1);
    old.state = State::Done;
    old.active = false;
    let file = serde_json::json!({ "schema": 1, "items": [old] });
    fs::write(
        storage.archive_path(2020),
        serde_json::to_vec(&file).unwrap(),
    )
    .unwrap();
    let loaded = storage.load().unwrap();
    assert_eq!(loaded.store.get(a).unwrap().title, "a");
    assert_eq!(loaded.store.items().len(), store.items().len());
}

#[test]
fn a_leftover_temporary_file_does_not_matter() {
    let (_dir, storage) = open();
    let (store, _) = sample();
    storage.save(&store).unwrap();
    // A crash while writing leaves a half-written temporary file; the original is untouched.
    fs::write(
        atomic::tmp_path(&storage.items_path()),
        b"{\"schema\": 1, \"items\": [{",
    )
    .unwrap();
    let loaded = storage.load().unwrap();
    assert_eq!(sorted(&loaded.store), sorted(&store));
    let mut changed = loaded.store;
    changed.resume().unwrap();
    storage.save(&changed).unwrap();
    assert!(!atomic::tmp_path(&storage.items_path()).exists());
}

#[test]
fn a_damaged_file_offers_the_backup() {
    let (_dir, storage) = open();
    let (mut store, [_, b, ..]) = sample();
    storage.save(&store).unwrap();
    store.finish(b, at(2026, 11, 22)).unwrap();
    storage.save(&store).unwrap();
    fs::write(storage.items_path(), b"{ kaputt").unwrap();

    let err = storage.load().unwrap_err();
    let LoadError::Corrupt { path, backup, .. } = &err else {
        panic!("expected Corrupt, got {err:?}");
    };
    assert_eq!(path, &storage.items_path());
    assert_eq!(
        backup.as_deref(),
        Some(backup_path(&storage.items_path()).as_path())
    );

    storage.restore_backup(path).unwrap();
    let loaded = storage.load().unwrap();
    assert!(
        loaded.store.get(b).is_some(),
        "the previous version is back"
    );
    assert_eq!(
        fs::read(atomic::broken_path(&storage.items_path())).unwrap(),
        b"{ kaputt"
    );
}

#[test]
fn a_damaged_file_does_not_replace_a_good_backup() {
    let (_dir, storage) = open();
    let (mut store, _) = sample();
    storage.save(&store).unwrap();
    store.resume().unwrap();
    storage.save(&store).unwrap();
    let good_backup = fs::read(backup_path(&storage.items_path())).unwrap();
    fs::write(storage.items_path(), b"{ kaputt").unwrap();
    store.pause().unwrap();
    storage.save(&store).unwrap();
    assert_eq!(
        fs::read(backup_path(&storage.items_path())).unwrap(),
        good_backup
    );
}

#[test]
fn a_file_from_a_newer_version_is_refused() {
    let (_dir, storage) = open();
    fs::write(storage.items_path(), br#"{ "schema": 7, "items": [] }"#).unwrap();
    assert!(matches!(
        storage.load(),
        Err(LoadError::TooNew { schema: 7, .. })
    ));
}

#[test]
fn a_hand_written_file_is_read_and_repaired() {
    let (_dir, storage) = open();
    let file = r#"{
      "items": [
        { "id": "01JD0000000000000000000001", "title": "Zweite", "state": "focus", "order": 5,
          "created_at": "2026-11-20T09:00:00Z", "updated_at": "2026-11-20T09:00:00Z" },
        { "id": "01JD0000000000000000000002", "title": "Erste", "state": "focus", "active": true,
          "order": 3, "created_at": "2026-11-20T09:00:00Z", "updated_at": "2026-11-20T09:00:00Z" },
        { "id": "01JD0000000000000000000003", "state": "idea", "active": true,
          "note": "- nur eine Notiz",
          "created_at": "2026-11-20T09:00:00Z", "updated_at": "2026-11-20T09:00:00Z" }
      ]
    }"#;
    fs::write(storage.items_path(), file).unwrap();
    let loaded = storage.load().unwrap();
    assert!(loaded.repaired);
    let focus: Vec<_> = loaded
        .store
        .list(State::Focus)
        .iter()
        .map(|it| it.title.clone())
        .collect();
    assert_eq!(focus, ["Erste", "Zweite"]);
    assert!(!loaded.store.list(State::Idea)[0].active);
    loaded.store.check_invariants().unwrap();
}

#[test]
fn saver_bundles_changes_and_flushes_on_drop() {
    let (_dir, storage) = open();
    let (mut store, [_, b, ..]) = sample();
    let saver = Saver::with_delay(storage.clone(), Duration::from_millis(150), |e| {
        panic!("save failed: {e}")
    });
    saver.save(store.clone());
    store.finish(b, at(2026, 11, 22)).unwrap();
    saver.save(store.clone());
    assert!(
        !storage.items_path().exists(),
        "nothing written before the delay"
    );
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(sorted(&storage.load().unwrap().store), sorted(&store));
    assert!(
        !backup_path(&storage.items_path()).exists(),
        "both changes in one write"
    );

    store.resume().unwrap();
    saver.save(store.clone());
    drop(saver);
    assert!(
        !storage.load().unwrap().store.on_break(),
        "pending change written on drop"
    );
}

#[test]
fn saver_flush_writes_at_once() {
    let (_dir, storage) = open();
    let (store, _) = sample();
    let saver = Saver::with_delay(storage.clone(), Duration::from_secs(60), |_| {});
    saver.save(store.clone());
    saver.flush().unwrap();
    assert_eq!(sorted(&storage.load().unwrap().store), sorted(&store));
}

#[test]
fn watcher_reports_foreign_changes_but_not_own_writes() {
    let (_dir, storage) = open();
    let (mut store, _) = sample();
    storage.save(&store).unwrap();
    let (tx, rx) = mpsc::channel();
    let _watcher = storage.watch(move |path| tx.send(path).unwrap()).unwrap();
    // Give the watcher a moment to register.
    std::thread::sleep(Duration::from_millis(100));

    store.resume().unwrap();
    storage.save(&store).unwrap();
    let id = add(&mut store, "neu", Target::Idea, at(2026, 11, 23));
    storage.save(&store).unwrap();
    store.finish(id, at(2024, 3, 3)).unwrap();
    storage.save(&store).unwrap();
    assert!(
        rx.recv_timeout(Duration::from_millis(700)).is_err(),
        "own writes must not be reported"
    );

    let mut raw: serde_json::Value =
        serde_json::from_slice(&fs::read(storage.items_path()).unwrap()).unwrap();
    raw["items"][0]["title"] = "von außen".into();
    fs::write(
        storage.items_path(),
        serde_json::to_vec_pretty(&raw).unwrap(),
    )
    .unwrap();
    let path = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("foreign change reported");
    assert_eq!(path, storage.items_path());

    let loaded = storage.load().unwrap();
    assert!(
        loaded
            .store
            .items()
            .iter()
            .any(|it| it.title == "von außen")
    );
}

#[test]
fn data_file_names() {
    let dir = Path::new("/d");
    assert!(is_data_file(dir, Path::new("/d/items.json")));
    assert!(is_data_file(dir, Path::new("/d/archive/2026.json")));
    assert!(!is_data_file(dir, Path::new("/d/items.json.tmp")));
    assert!(!is_data_file(dir, Path::new("/d/items.json.bak")));
    assert!(!is_data_file(dir, Path::new("/d/archive/2026.json.bak")));
    assert!(!is_data_file(dir, Path::new("/d/archive/notes.json")));
}
