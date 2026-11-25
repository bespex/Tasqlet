//! The popover window: prepared hidden at startup, then only shown and hidden, never recreated
//! (docs/KONZEPT.md 1.1, "Instant und leicht"). Showing emits `popover-shown`, so the webview can
//! refresh. Behaviour and placement follow Maillet's popup (`../Maillet/src-tauri/src/app/windows.rs`):
//!
//! - X11: next to the tray icon, above it if the panel is at the bottom, below it otherwise.
//! - Wayland: compositors ignore positions; Tasqlet asks for the top right corner of the screen
//!   under the mouse and the compositor decides (choice of corner or centre comes with the
//!   settings, plan 2.6).
//! - Losing the focus hides it. A click on the tray icon first takes the focus away, so a click
//!   right after hiding does not show it again.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent};

pub const LABEL: &str = "popover";
/// Event to the webview: put the cursor into the quick entry.
pub const FOCUS_CAPTURE: &str = "focus-capture";
/// Gap between the popover and the tray icon or the screen edge.
const MARGIN: i32 = 8;
/// Events within this time after showing or hiding belong to that same click.
const SAME_CLICK: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct PopoverState {
    /// Where the tray icon was last clicked (X11 only reports it).
    anchor: Mutex<Option<(i32, i32)>>,
    shown_at: Mutex<Option<Instant>>,
    hidden_on_blur: Mutex<Option<Instant>>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn recent(at: &Mutex<Option<Instant>>) -> bool {
    lock(at).is_some_and(|t| t.elapsed() < SAME_CLICK)
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Hide on focus loss, hide instead of close.
pub fn setup(app: &AppHandle) {
    app.manage(PopoverState::default());
    let Some(w) = window(app) else { return };
    let handle = app.clone();
    w.on_window_event(move |event| match event {
        WindowEvent::Focused(false) => {
            let state = handle.state::<PopoverState>();
            if !recent(&state.shown_at) {
                *lock(&state.hidden_on_blur) = Some(Instant::now());
                hide(&handle);
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(&handle);
        }
        _ => {}
    });
}

/// Tray click or `--toggle`: show, or hide if it is open.
pub fn toggle(app: &AppHandle, at: Option<(i32, i32)>) {
    let Some(w) = window(app) else { return };
    // Clicking the icon first takes the focus away, which already hid the popover.
    let just_hidden = recent(&app.state::<PopoverState>().hidden_on_blur);
    if w.is_visible().unwrap_or(false) {
        hide(app);
    } else if !just_hidden {
        show(app, at);
    }
}

/// Show the popover, at the tray icon if its position is known.
pub fn show(app: &AppHandle, at: Option<(i32, i32)>) {
    let Some(w) = window(app) else { return };
    let state = app.state::<PopoverState>();
    if let Some(p) = at {
        *lock(&state.anchor) = Some(p);
    }
    *lock(&state.shown_at) = Some(Instant::now());
    // GTK ignores moves of hidden windows: place, show, place again.
    place(app, &w);
    let _ = w.show();
    place(app, &w);
    settle(app, &w);
    let _ = w.set_always_on_top(true);
    let _ = w.set_focus();
    activate_x11(&w);
    let _ = app.emit_to(LABEL, "popover-shown", ());
}

/// The first time the window is mapped, its size (and the window manager's frame) is only known
/// a moment later, so the first placement can be off. Place again once it has settled.
fn settle(app: &AppHandle, w: &WebviewWindow) {
    let (app, w) = (app.clone(), w.clone());
    std::thread::spawn(move || {
        for delay in [40, 120] {
            std::thread::sleep(Duration::from_millis(delay));
            if !w.is_visible().unwrap_or(false) {
                return;
            }
            let (app, w) = (app.clone(), w.clone());
            let _ = app.clone().run_on_main_thread(move || place(&app, &w));
        }
    });
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = window(app) {
        let _ = w.hide();
    }
}

#[tauri::command]
pub fn hide_popover(app: AppHandle) {
    hide(&app);
}

/// Where the popover goes: next to the anchor if known, else the top right of the work area.
/// All values in physical pixels.
pub fn position(
    anchor: Option<(i32, i32)>,
    area: (i32, i32, i32, i32),
    size: (i32, i32),
) -> (i32, i32) {
    let (ax, ay, aw, ah) = area;
    let (width, height) = size;
    let (x, y) = match anchor {
        Some((cx, cy)) => (
            cx - width / 2,
            if cy > ay + ah / 2 {
                cy - height - MARGIN
            } else {
                cy + MARGIN
            },
        ),
        None => (ax + aw - width - MARGIN, ay + MARGIN),
    };
    // Keep it on screen; a popover larger than the screen sticks to the top left.
    let fit = |v: i32, lo: i32, hi: i32| v.min(hi).max(lo);
    (
        fit(x, ax + MARGIN, ax + aw - width - MARGIN),
        fit(y, ay + MARGIN, ay + ah - height - MARGIN),
    )
}

fn place(app: &AppHandle, w: &WebviewWindow) {
    let anchor = if crate::platform::is_x11() {
        *lock(&app.state::<PopoverState>().anchor)
    } else {
        None
    };
    let point = anchor.or_else(|| app.cursor_position().ok().map(|p| (p.x as i32, p.y as i32)));
    let monitor = point
        .and_then(|(x, y)| w.monitor_from_point(x as f64, y as f64).ok().flatten())
        .or_else(|| w.primary_monitor().ok().flatten());
    let (Some(monitor), Ok(size)) = (monitor, w.outer_size()) else {
        return;
    };
    let area = monitor.work_area();
    let (x, y) = position(
        anchor,
        (
            area.position.x,
            area.position.y,
            area.size.width as i32,
            area.size.height as i32,
        ),
        (size.width as i32, size.height as i32),
    );
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

/// Window managers refuse the focus to windows of background programs, and a tray click does not
/// count as a user action for our window. Without the focus the popover would not hide when
/// clicking elsewhere. Pagers and task bars may activate windows (`_NET_ACTIVE_WINDOW` with
/// source 2); Tasqlet does the same, with `x11rb` instead of Xlib (no `unsafe`). The first request
/// can come before the window is mapped, so it is repeated briefly unless it worked.
fn activate_x11(w: &WebviewWindow) {
    if !crate::platform::is_x11() {
        return;
    }
    let title = w.title().unwrap_or_default();
    let w = w.clone();
    std::thread::spawn(move || {
        for delay in [0, 150, 250, 400, 700] {
            std::thread::sleep(Duration::from_millis(delay));
            if !w.is_visible().unwrap_or(false) || w.is_focused().unwrap_or(false) {
                return;
            }
            crate::platform::activate_window(&title);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: (i32, i32, i32, i32) = (0, 0, 1920, 1080);
    const SIZE: (i32, i32) = (470, 680);

    #[test]
    fn above_an_icon_in_a_bottom_panel() {
        assert_eq!(position(Some((1800, 1070)), AREA, SIZE), (1442, 382));
    }

    #[test]
    fn below_an_icon_in_a_top_panel() {
        assert_eq!(position(Some((1000, 10)), AREA, SIZE), (765, 18));
    }

    #[test]
    fn top_right_corner_without_anchor() {
        assert_eq!(position(None, AREA, SIZE), (1442, 8));
        // Second monitor to the right.
        assert_eq!(position(None, (1920, 0, 2560, 1440), SIZE), (4002, 8));
    }

    #[test]
    fn stays_on_a_small_screen() {
        assert_eq!(position(Some((5, 5)), (0, 0, 800, 600), SIZE), (8, 8));
        assert_eq!(position(None, (0, 0, 400, 600), SIZE), (8, 8));
    }
}
