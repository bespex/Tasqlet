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
//! - Form **sidebar** (setting, D41): 700 px wide, the full height of the work area, at the edge of
//!   the screen half with the tray icon (or the edge the user picked). It stays when it loses the
//!   focus, but is kept above other windows only when pinned; tray click, key or `--toggle` bring
//!   it back to the front. The choice holds for tray click, global key and `--toggle` alike.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tasqlet_core::settings::{SidebarSide, WindowForm};
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
    /// Without an icon position: the middle of the screen instead of the top right corner.
    center: AtomicBool,
    /// The elaboration is open: wide, and always in the middle of the screen.
    wide: AtomicBool,
    /// Pinned: stays when it loses the focus, e.g. to drop a file from the file manager (D37).
    pinned: AtomicBool,
    /// Form sidebar instead of popover (D41).
    sidebar: AtomicBool,
    side: Mutex<SidebarSide>,
    /// Whether the tray icon was last seen in the left half of its screen.
    tray_left: Mutex<Option<bool>>,
    /// The sidebar went behind other windows when it lost the focus, at this moment.
    lowered: Mutex<Option<Instant>>,
}

/// Popover or sidebar, and the edge of the sidebar (a setting, D41). An open window changes its form
/// in place and tells the webview (which offers pinning only in the popover).
pub fn set_form(app: &AppHandle, form: WindowForm, side: SidebarSide) {
    let state = app.state::<PopoverState>();
    state
        .sidebar
        .store(form == WindowForm::Sidebar, Ordering::Relaxed);
    *lock(&state.side) = side;
    if let Some(w) = window(app).filter(|w| w.is_visible().unwrap_or(false)) {
        state.pinned.store(false, Ordering::Relaxed);
        let _ = w.set_always_on_top(form == WindowForm::Popover);
        let wide = state.wide.load(Ordering::Relaxed);
        resize(app, &w, wide);
        place(app, &w);
        settle(app, &w);
        let _ = app.emit_to(LABEL, "popover-shown", ());
    }
}

fn is_sidebar(app: &AppHandle) -> bool {
    app.state::<PopoverState>().sidebar.load(Ordering::Relaxed)
}

/// Where the popover goes without an icon position (a setting, plan 2.6).
pub fn set_center(app: &AppHandle, center: bool) {
    app.state::<PopoverState>()
        .center
        .store(center, Ordering::Relaxed);
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
            let pinned = state.pinned.load(Ordering::Relaxed);
            if state.sidebar.load(Ordering::Relaxed) {
                // The sidebar stays; the window that took the focus may now cover it.
                if !recent(&state.shown_at) && !pinned {
                    *lock(&state.lowered) = Some(Instant::now());
                }
            // Open settings keep it open, so their changes show here at once (D40)
            } else if !recent(&state.shown_at)
                && !pinned
                && !crate::settings_window::is_open(&handle)
            {
                *lock(&state.hidden_on_blur) = Some(Instant::now());
                hide(&handle);
            }
        }
        // A click into the sidebar brings it to the front again.
        WindowEvent::Focused(true) => {
            lock(&handle.state::<PopoverState>().lowered).take();
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
    let state = app.state::<PopoverState>();
    let just_hidden = recent(&state.hidden_on_blur);
    // A sidebar behind other windows comes to the front; one that was in front a moment ago (this
    // very click took its focus) hides.
    let behind = lock(&state.lowered).is_some_and(|t| t.elapsed() >= SAME_CLICK);
    let visible = w.is_visible().unwrap_or(false);
    if visible && is_sidebar(app) && behind {
        show(app, at);
    } else if visible {
        hide(app);
    } else if !just_hidden {
        show(app, at);
    }
}

/// Global key and `--toggle`: like the tray click, except that a pinned sidebar is never hidden by
/// them, only brought to the front with the focus (it is kept above other windows anyway).
pub fn toggle_by_key(app: &AppHandle) {
    let Some(w) = window(app) else { return };
    let pinned = app.state::<PopoverState>().pinned.load(Ordering::Relaxed);
    if w.is_visible().unwrap_or(false) && is_sidebar(app) && pinned {
        let _ = w.set_focus();
        activate_x11(&w);
    } else {
        toggle(app, None);
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
    *lock(&state.lowered) = None;
    // The sidebar takes the height of the screen it opens on.
    resize(app, &w, state.wide.load(Ordering::Relaxed));
    // GTK ignores moves of hidden windows: place, show, place again.
    place(app, &w);
    let _ = w.show();
    place(app, &w);
    settle(app, &w);
    // Above other windows: the popover always, the sidebar only when pinned.
    let _ = w.set_always_on_top(!is_sidebar(app) || state.pinned.load(Ordering::Relaxed));
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

/// The settings closed: if the popover is still open, it gets the focus back, so that the next click
/// elsewhere or Esc hides it as usual.
pub fn refocus(app: &AppHandle) {
    let Some(w) = window(app) else { return };
    if w.is_visible().unwrap_or(false) {
        *lock(&app.state::<PopoverState>().shown_at) = Some(Instant::now());
        let _ = w.set_focus();
        activate_x11(&w);
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = window(app) {
        // The next time it opens, it is the normal popover again, not the elaboration, not pinned.
        let state = app.state::<PopoverState>();
        state.wide.store(false, Ordering::Relaxed);
        state.pinned.store(false, Ordering::Relaxed);
        resize(app, &w, false);
        let _ = w.hide();
        let _ = app.emit_to(LABEL, "popover-hidden", ());
    }
}

/// Width of the popover, normal (as wide as Maillet's window, D39) and with the elaboration beside the
/// lists (plan 2.4, `KONZEPT.md` 6.5).
const WIDTH: f64 = 700.0;
const WIDTH_WIDE: f64 = 940.0;

/// Height of the normal popover.
const HEIGHT: f64 = 680.0;

/// The monitor the popover is on (or would appear on).
fn monitor_of(app: &AppHandle, w: &WebviewWindow) -> Option<tauri::Monitor> {
    let anchor = *lock(&app.state::<PopoverState>().anchor);
    let point = anchor.or_else(|| app.cursor_position().ok().map(|p| (p.x as i32, p.y as i32)));
    point
        .and_then(|(x, y)| w.monitor_from_point(x as f64, y as f64).ok().flatten())
        .or_else(|| w.primary_monitor().ok().flatten())
}

/// The size of the elaboration: roomy, but never larger than the screen allows (logical pixels).
pub fn wide_size(work_area: (f64, f64)) -> (f64, f64) {
    let (aw, ah) = work_area;
    let width = (aw * 0.75)
        .clamp(WIDTH_WIDE, 1280.0)
        .min(aw - 2.0 * MARGIN as f64);
    let height = (ah * 0.88)
        .clamp(HEIGHT, 900.0)
        .min(ah - 2.0 * MARGIN as f64);
    (width, height)
}

/// Resize to the normal popover or the roomy elaboration; as a sidebar always the full height.
fn resize(app: &AppHandle, w: &WebviewWindow, wide: bool) {
    let work = monitor_of(app, w).map(|m| {
        let scale = m.scale_factor();
        let area = m.work_area();
        (
            area.size.width as f64 / scale,
            area.size.height as f64 / scale,
        )
    });
    let mut size = match (wide, work) {
        (true, Some(work)) => wide_size(work),
        (true, None) => (WIDTH_WIDE, HEIGHT),
        (false, _) => (WIDTH, HEIGHT),
    };
    if let (true, Some((_, height))) = (is_sidebar(app), work) {
        size.1 = height;
    }
    let _ = w.set_size(tauri::LogicalSize::new(size.0, size.1));
}

/// Make the popover roomy for the elaboration, always in the middle of the screen, or normal again
/// (back at the icon). Only a real change moves the window: navigating inside the elaboration, or a
/// window the user has dragged elsewhere, stays where it is.
#[tauri::command]
pub fn set_wide(app: AppHandle, wide: bool) {
    let Some(w) = window(&app) else { return };
    if app
        .state::<PopoverState>()
        .wide
        .swap(wide, Ordering::Relaxed)
        == wide
    {
        return;
    }
    resize(&app, &w, wide);
    // The window manager applies the size a moment later: place again afterwards.
    place(&app, &w);
    settle(&app, &w);
}

/// Pin or unpin: pinned, the popover stays open when another window gets the focus (D37) and the
/// sidebar is kept above other windows (D41); unpinned, other windows may cover the sidebar.
#[tauri::command]
pub fn set_pinned(app: AppHandle, pinned: bool) {
    let state = app.state::<PopoverState>();
    state.pinned.store(pinned, Ordering::Relaxed);
    if is_sidebar(&app) {
        if pinned {
            *lock(&state.lowered) = None;
        }
        if let Some(w) = window(&app) {
            let _ = w.set_always_on_top(pinned);
        }
    }
}

#[tauri::command]
pub fn hide_popover(app: AppHandle) {
    hide(&app);
}

/// Which edge the sidebar takes: the chosen one, or the half of the screen with the tray icon (last
/// seen), else the right.
pub fn sidebar_left(side: SidebarSide, tray_left: Option<bool>) -> bool {
    match side {
        SidebarSide::Left => true,
        SidebarSide::Right => false,
        SidebarSide::Auto => tray_left.unwrap_or(false),
    }
}

/// The sidebar's top left corner: at the left or right edge of the work area, from its top. The
/// elaboration grows from that edge towards the middle. Physical pixels.
pub fn sidebar_position(area: (i32, i32, i32, i32), width: i32, left: bool) -> (i32, i32) {
    let (ax, ay, aw, _) = area;
    (if left { ax } else { ax + aw - width.min(aw) }, ay)
}

/// Where the popover goes: next to the anchor if known, else the top right of the work area.
/// All values in physical pixels.
pub fn position(
    anchor: Option<(i32, i32)>,
    area: (i32, i32, i32, i32),
    size: (i32, i32),
    center: bool,
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
        None if center => (ax + (aw - width) / 2, ay + (ah - height) / 2),
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
    if is_sidebar(app) {
        return place_sidebar(app, w);
    }
    let wide = app.state::<PopoverState>().wide.load(Ordering::Relaxed);
    let anchor = if crate::platform::is_x11() && !wide {
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
        wide || app.state::<PopoverState>().center.load(Ordering::Relaxed),
    );
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

fn place_sidebar(app: &AppHandle, w: &WebviewWindow) {
    let state = app.state::<PopoverState>();
    let anchor = *lock(&state.anchor);
    let point = anchor.or_else(|| app.cursor_position().ok().map(|p| (p.x as i32, p.y as i32)));
    let monitor = point
        .and_then(|(x, y)| w.monitor_from_point(x as f64, y as f64).ok().flatten())
        .or_else(|| w.primary_monitor().ok().flatten());
    let (Some(monitor), Ok(size)) = (monitor, w.outer_size()) else {
        return;
    };
    let area = monitor.work_area();
    let area = (
        area.position.x,
        area.position.y,
        area.size.width as i32,
        area.size.height as i32,
    );
    // Seen from a tray click (X11): in which half of its screen the icon sits.
    if let (true, Some((cx, _))) = (crate::platform::is_x11(), anchor) {
        *lock(&state.tray_left) = Some(cx < area.0 + area.2 / 2);
    }
    let left = sidebar_left(*lock(&state.side), *lock(&state.tray_left));
    let (x, y) = sidebar_position(area, size.width as i32, left);
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
    const SIZE: (i32, i32) = (700, 680);

    #[test]
    fn the_sidebar_takes_the_chosen_edge_or_the_side_of_the_icon() {
        assert!(sidebar_left(SidebarSide::Left, Some(false)));
        assert!(!sidebar_left(SidebarSide::Right, Some(true)));
        assert!(sidebar_left(SidebarSide::Auto, Some(true)));
        assert!(!sidebar_left(SidebarSide::Auto, None));
        assert_eq!(sidebar_position((0, 30, 1920, 1050), 700, true), (0, 30));
        assert_eq!(
            sidebar_position((0, 30, 1920, 1050), 700, false),
            (1220, 30)
        );
        assert_eq!(
            sidebar_position((1920, 0, 1280, 1024), 1400, false),
            (1920, 0)
        );
    }

    #[test]
    fn above_an_icon_in_a_bottom_panel() {
        assert_eq!(position(Some((1800, 1070)), AREA, SIZE, false), (1212, 382));
    }

    #[test]
    fn below_an_icon_in_a_top_panel() {
        assert_eq!(position(Some((1000, 10)), AREA, SIZE, false), (650, 18));
    }

    #[test]
    fn top_right_corner_without_anchor() {
        assert_eq!(position(None, AREA, SIZE, false), (1212, 8));
        // Second monitor to the right.
        assert_eq!(
            position(None, (1920, 0, 2560, 1440), SIZE, false),
            (3772, 8)
        );
    }

    #[test]
    fn the_elaboration_is_roomy_but_fits_the_screen() {
        // A big screen: capped, not stretched over everything.
        assert_eq!(wide_size((2560.0, 1400.0)), (1280.0, 900.0));
        // A laptop: three quarters of the width, but at least the normal height.
        assert_eq!(wide_size((1366.0, 728.0)), (1024.5, 680.0));
        // A small screen: smaller than the preferred size, still inside the work area.
        let (w, h) = wide_size((800.0, 600.0));
        assert!(w <= 784.0 && h <= 584.0);
    }

    #[test]
    fn the_middle_of_the_screen_when_asked_for() {
        assert_eq!(position(None, AREA, SIZE, true), (610, 200));
        // An icon position still wins.
        assert_eq!(position(Some((1000, 10)), AREA, SIZE, true), (650, 18));
        // Small screens: still on screen.
        assert_eq!(position(None, (0, 0, 400, 600), SIZE, true), (8, 8));
    }

    #[test]
    fn stays_on_a_small_screen() {
        assert_eq!(
            position(Some((5, 5)), (0, 0, 800, 600), SIZE, false),
            (8, 8)
        );
        assert_eq!(position(None, (0, 0, 400, 600), SIZE, false), (8, 8));
    }
}
