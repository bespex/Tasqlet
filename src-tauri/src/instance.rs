//! One running Tasqlet per data folder, reachable over a Unix socket (pattern of CodeDeck's hub).
//!
//! The first start takes a lock file and listens on a socket next to it, both in
//! `$XDG_RUNTIME_DIR/tasqlet/`, named after the data folder. A second start finds the lock taken
//! and hands its request to the first one instead: `tasqlet --toggle` shows or hides the popover,
//! plain `tasqlet` shows it. This is what a desktop binds to a key where programs cannot hold
//! global shortcuts (Wayland, D10). The CLI of plan step 2.5 extends the same socket.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A request to the running instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "cmd")]
pub enum Request {
    /// Show the popover (plain second start).
    Show,
    /// Show the popover, or hide it if it is open (`--toggle`).
    Toggle,
}

/// Outcome of [`claim`].
pub enum Start {
    /// This is the only instance; keep the value alive as long as the app runs.
    First(Instance),
    /// Another instance runs already.
    Running,
}

/// The lock and socket of the running instance. Dropping it removes the socket.
pub struct Instance {
    _lock: File,
    socket: PathBuf,
}

/// Where lock and socket live: `$XDG_RUNTIME_DIR/tasqlet`, else a private folder in the temp dir.
pub fn runtime_dir() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join("tasqlet"),
        _ => std::env::temp_dir().join(format!("tasqlet-{}", uid())),
    }
}

/// The owner of our own process folder is our user id; no libc needed for it.
fn uid() -> u32 {
    fs::metadata("/proc/self").map_or(0, |m| m.uid())
}

/// Lock and socket names for a data folder, so instances on different folders (tests,
/// development with `TASQLET_DATA_DIR`) do not get in each other's way.
pub fn key(data_dir: &Path) -> String {
    let mut h = DefaultHasher::new();
    data_dir.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Try to become the instance for `key`.
pub fn claim(dir: &Path, key: &str) -> io::Result<Start> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(format!("{key}.lock")))?;
    match lock.try_lock() {
        Ok(()) => Ok(Start::First(Instance {
            _lock: lock,
            socket: dir.join(format!("{key}.sock")),
        })),
        Err(TryLockError::WouldBlock) => Ok(Start::Running),
        Err(TryLockError::Error(e)) => Err(e),
    }
}

impl Instance {
    /// Answer requests from now on; `handle` runs on a background thread.
    pub fn listen(&self, handle: impl Fn(Request) + Send + Sync + 'static) -> io::Result<()> {
        // Left over from a crash; the lock says it is ours now.
        let _ = fs::remove_file(&self.socket);
        let listener = UnixListener::bind(&self.socket)?;
        fs::set_permissions(&self.socket, fs::Permissions::from_mode(0o600))?;
        std::thread::Builder::new()
            .name("tasqlet-instance".into())
            .spawn(move || {
                for stream in listener.incoming().flatten() {
                    serve(stream, &handle);
                }
            })?;
        Ok(())
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
    }
}

fn serve(mut stream: UnixStream, handle: &impl Fn(Request)) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut text = String::new();
    if (&mut stream)
        .take(64 * 1024)
        .read_to_string(&mut text)
        .is_err()
    {
        return;
    }
    let reply = match serde_json::from_str::<Request>(&text) {
        Ok(request) => {
            handle(request);
            r#"{"ok":true}"#
        }
        Err(_) => r#"{"error":"unknown request"}"#,
    };
    let _ = stream.write_all(reply.as_bytes());
}

/// Hand a request to the running instance. It may still be starting, so a missing socket is
/// retried for a moment.
pub fn send(dir: &Path, key: &str, request: Request) -> io::Result<()> {
    let socket = dir.join(format!("{key}.sock"));
    let mut last = None;
    for _ in 0..20 {
        match UnixStream::connect(&socket) {
            Ok(mut stream) => {
                stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                stream.write_all(serde_json::to_string(&request)?.as_bytes())?;
                stream.shutdown(std::net::Shutdown::Write)?;
                let mut answer = String::new();
                stream.read_to_string(&mut answer)?;
                return if answer.contains("\"ok\"") {
                    Ok(())
                } else {
                    Err(io::Error::other(answer))
                };
            }
            Err(e) => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("no running instance")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn second_start_hands_its_request_to_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let key = key(Path::new("/some/data"));
        let Start::First(instance) = claim(dir.path(), &key).unwrap() else {
            panic!("first claim must succeed");
        };
        let (tx, rx) = mpsc::channel();
        let tx = std::sync::Mutex::new(tx);
        instance
            .listen(move |r| tx.lock().unwrap().send(r).unwrap())
            .unwrap();

        assert!(matches!(claim(dir.path(), &key).unwrap(), Start::Running));
        send(dir.path(), &key, Request::Toggle).unwrap();
        send(dir.path(), &key, Request::Show).unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            Request::Toggle
        );
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            Request::Show
        );

        drop(instance);
        assert!(!dir.path().join(format!("{key}.sock")).exists());
        assert!(matches!(claim(dir.path(), &key).unwrap(), Start::First(_)));
    }

    #[test]
    fn different_data_folders_get_different_keys() {
        assert_ne!(key(Path::new("/a")), key(Path::new("/b")));
        assert_eq!(key(Path::new("/a")), key(Path::new("/a")));
    }

    #[test]
    fn requests_have_a_small_wire_format() {
        assert_eq!(
            serde_json::to_string(&Request::Toggle).unwrap(),
            r#"{"cmd":"toggle"}"#
        );
    }
}
