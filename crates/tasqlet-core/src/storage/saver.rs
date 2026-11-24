//! Write-behind: changes are collected and written at most [`SAVE_DELAY`] after the first unsaved
//! change, in a background thread. Continuous typing still gets saved regularly, and a burst of
//! actions becomes one write.

use std::io;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::Storage;
use crate::store::Store;

/// How long changes are collected before they are written (`docs/KONZEPT.md` 2.5).
pub const SAVE_DELAY: Duration = Duration::from_millis(300);

type ErrorHandler = Box<dyn Fn(io::Error) + Send + Sync>;

/// Saves the latest state of a store in the background. Dropping it writes what is pending.
pub struct Saver {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

struct Shared {
    storage: Storage,
    delay: Duration,
    pending: Mutex<Pending>,
    wake: Condvar,
    /// Held while taking the pending state and writing it, so writes happen in order.
    writer: Mutex<()>,
    on_error: ErrorHandler,
}

#[derive(Default)]
struct Pending {
    store: Option<Store>,
    due: Option<Instant>,
    shutdown: bool,
}

impl Saver {
    /// A saver writing to `storage` after [`SAVE_DELAY`]. Errors of background writes go to
    /// `on_error`.
    pub fn new(storage: Storage, on_error: impl Fn(io::Error) + Send + Sync + 'static) -> Self {
        Self::with_delay(storage, SAVE_DELAY, on_error)
    }

    /// Like [`Saver::new`] with another delay.
    pub fn with_delay(
        storage: Storage,
        delay: Duration,
        on_error: impl Fn(io::Error) + Send + Sync + 'static,
    ) -> Self {
        let shared = Arc::new(Shared {
            storage,
            delay,
            pending: Mutex::new(Pending::default()),
            wake: Condvar::new(),
            writer: Mutex::new(()),
            on_error: Box::new(on_error),
        });
        let worker = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("tasqlet-saver".into())
            .spawn(move || run(&worker))
            .expect("spawn saver thread");
        Self {
            shared,
            thread: Some(thread),
        }
    }

    /// Schedule this state to be written. A later call before the write replaces it.
    pub fn save(&self, store: Store) {
        let mut p = lock(&self.shared.pending);
        p.store = Some(store);
        if p.due.is_none() {
            p.due = Some(Instant::now() + self.shared.delay);
        }
        self.shared.wake.notify_all();
    }

    /// Write what is pending now, e.g. before quitting.
    pub fn flush(&self) -> io::Result<()> {
        write_pending(&self.shared)
    }
}

impl Drop for Saver {
    fn drop(&mut self) {
        lock(&self.shared.pending).shutdown = true;
        self.shared.wake.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(shared: &Shared) {
    loop {
        let shutdown = {
            let mut p = lock(&shared.pending);
            loop {
                if p.shutdown {
                    break;
                }
                match p.due {
                    None => p = shared.wake.wait(p).unwrap_or_else(|e| e.into_inner()),
                    Some(due) => {
                        let now = Instant::now();
                        if now >= due {
                            break;
                        }
                        p = shared
                            .wake
                            .wait_timeout(p, due - now)
                            .unwrap_or_else(|e| e.into_inner())
                            .0;
                    }
                }
            }
            p.shutdown
        };
        if let Err(e) = write_pending(shared) {
            (shared.on_error)(e);
        }
        if shutdown {
            break;
        }
    }
}

fn write_pending(shared: &Shared) -> io::Result<()> {
    let _writer = lock(&shared.writer);
    let store = {
        let mut p = lock(&shared.pending);
        p.due = None;
        p.store.take()
    };
    match store {
        Some(store) => shared.storage.save(&store),
        None => Ok(()),
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
