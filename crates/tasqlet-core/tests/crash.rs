//! Acceptance of plan step 1.2: data survive a `kill -9` in the middle of writing.
//!
//! The test starts its own test binary again as a child process that saves in a tight loop
//! (adding items and moving them to the archive, so both phases of a save are exercised), kills
//! it with SIGKILL, half of the time at varying moments and half of the time exactly while a
//! temporary file exists (between writing and renaming), and checks that the data load and that no item was lost.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Duration as Span, TimeZone, Utc};
use tasqlet_core::storage::Storage;
use tasqlet_core::{NewItem, Store, Target};

const CHILD_ENV: &str = "TASQLET_CRASH_TEST_DIR";
const TEST_NAME: &str = "data_survive_kill_during_writes";

#[test]
fn data_survive_kill_during_writes() {
    if let Some(dir) = std::env::var_os(CHILD_ENV) {
        write_forever(&Storage::open(dir).unwrap());
    }

    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::open(dir.path()).unwrap();
    let mut store = Store::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap();
    for i in 0..1500 {
        let mut item = NewItem::new(format!("Eintrag {i}"));
        item.note = "Ein wenig Text, damit die Datei groß genug ist. ".repeat(4);
        store.add(item, Target::Idea, start).unwrap();
    }
    storage.save(&store).unwrap();

    let mut count = store.items().len();
    let mut caught_mid_write = 0;
    for round in 0..16u64 {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([TEST_NAME, "--exact", "--test-threads=1"])
            .env(CHILD_ENV, dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        if round % 2 == 0 {
            // At a varying moment.
            thread::sleep(Duration::from_millis(150 + (round * 37) % 200));
        } else {
            // While a temporary file exists, i.e. between writing it and renaming it.
            let deadline = Instant::now() + Duration::from_secs(5);
            while temp_files(dir.path()) == 0 && Instant::now() < deadline {
                thread::sleep(Duration::from_micros(100));
            }
        }
        child.kill().unwrap(); // SIGKILL on Unix
        child.wait().unwrap();
        if temp_files(dir.path()) > 0 {
            caught_mid_write += 1;
        }

        let loaded = Storage::open(dir.path())
            .unwrap()
            .load()
            .unwrap_or_else(|e| panic!("round {round}: data no longer load: {e}"));
        loaded.store.check_invariants().unwrap();
        let now = loaded.store.items().len();
        assert!(
            now >= count,
            "round {round}: items went missing ({count} before, {now} after)"
        );
        count = now;
    }
    assert!(
        count > 1500,
        "the child processes should have saved something"
    );
    assert!(
        caught_mid_write >= 2,
        "only {caught_mid_write} kills hit a write in progress"
    );
}

/// Temporary files left in the data folder and its archive.
fn temp_files(dir: &Path) -> usize {
    [dir.to_path_buf(), dir.join("archive")]
        .iter()
        .filter_map(|d| fs::read_dir(d).ok())
        .flatten()
        .filter(|e| {
            e.as_ref()
                .is_ok_and(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        })
        .count()
}

/// Child process: keep changing and saving until killed.
fn write_forever(storage: &Storage) -> ! {
    let mut store = storage.load().unwrap().store;
    let mut now = Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap();
    loop {
        now += Span::days(13);
        let item = NewItem::new("vom Kindprozess");
        let id = item.id;
        store.add(item, Target::Focus, now).unwrap();
        storage.save(&store).unwrap();
        // Finishing moves the item into an archive year: a two-phase save.
        store.finish(id, now).unwrap();
        storage.save(&store).unwrap();
    }
}
