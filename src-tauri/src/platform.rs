//! What differs between desktops and display servers, kept in one place (CLAUDE.md: no dependency
//! on a distribution, display server or desktop).

use std::sync::OnceLock;

/// Whether the windows of this process are X11 windows (X11 session, or GTK forced onto XWayland).
pub fn is_x11() -> bool {
    static X11: OnceLock<bool> = OnceLock::new();
    *X11.get_or_init(|| {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        x11_from(
            &var("GDK_BACKEND"),
            &var("XDG_SESSION_TYPE"),
            &var("WAYLAND_DISPLAY"),
            &var("DISPLAY"),
        )
    })
}

fn x11_from(gdk_backend: &str, session_type: &str, wayland_display: &str, display: &str) -> bool {
    if gdk_backend.starts_with("x11") {
        return true;
    }
    if gdk_backend.starts_with("wayland") {
        return false;
    }
    let wayland = session_type == "wayland" || !wayland_display.is_empty();
    !wayland && !display.is_empty()
}

/// Ask the window manager to activate the window of this process with this title, the way a pager
/// does (`_NET_ACTIVE_WINDOW`, source 2). Pattern of CodeDeck's `window.rs`. X11 only.
pub fn activate_window(title: &str) -> Option<()> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask};
    let (conn, screen) = x11rb::connect(None).ok()?;
    let root = conn.setup().roots.get(screen)?.root;
    let atom = |name: &str| {
        conn.intern_atom(false, name.as_bytes())
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    };
    let (clients, pid_atom, active) = (
        atom("_NET_CLIENT_LIST")?,
        atom("_NET_WM_PID")?,
        atom("_NET_ACTIVE_WINDOW")?,
    );
    let (name_atom, utf8) = (atom("_NET_WM_NAME")?, atom("UTF8_STRING")?);
    let list = conn
        .get_property(false, root, clients, AtomEnum::WINDOW, 0, 4096)
        .ok()?
        .reply()
        .ok()?;
    let me = std::process::id();
    let mine: Vec<u32> = list
        .value32()?
        .filter(|&w| {
            conn.get_property(false, w, pid_atom, AtomEnum::CARDINAL, 0, 1)
                .ok()
                .and_then(|c| c.reply().ok())
                .and_then(|r| r.value32().and_then(|mut v| v.next()))
                == Some(me)
        })
        .collect();
    let ours = mine.iter().copied().find(|&w| {
        conn.get_property(false, w, name_atom, utf8, 0, 1024)
            .ok()
            .and_then(|c| c.reply().ok())
            .is_some_and(|r| r.value == title.as_bytes())
    });
    let ours = ours.or_else(|| (mine.len() == 1).then(|| mine[0]))?;
    let event = ClientMessageEvent::new(32, ours, active, [2, x11rb::CURRENT_TIME, 0, 0, 0]);
    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .ok()?;
    conn.flush().ok()
}

/// Whether a tray host can show the icon: someone owns `org.kde.StatusNotifierWatcher` on the
/// session bus. GNOME without the AppIndicator extension has none. (Asking ksni whether a host is
/// registered is not reliable: Cinnamon's xapp-sn-watcher shows icons while reporting none.)
pub async fn tray_host_available() -> bool {
    let Ok(conn) = zbus::Connection::session().await else {
        return false;
    };
    let Ok(dbus) = zbus::fdo::DBusProxy::new(&conn).await else {
        return false;
    };
    let Ok(name) = zbus::names::BusName::try_from("org.kde.StatusNotifierWatcher") else {
        return false;
    };
    dbus.name_has_owner(name).await.unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_server_detection() {
        assert!(x11_from("", "x11", "", ":0"));
        assert!(!x11_from("", "wayland", "wayland-0", ":0"));
        assert!(!x11_from("", "", "wayland-0", ":0"));
        assert!(
            x11_from("x11", "wayland", "wayland-0", ":0"),
            "GTK forced onto XWayland"
        );
        assert!(!x11_from("wayland", "x11", "", ":0"));
        assert!(!x11_from("", "tty", "", ""));
    }
}
