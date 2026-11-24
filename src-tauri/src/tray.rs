//! The tray icon via StatusNotifierItem (ksni), as in Maillet and CodeDeck: unlike libappindicator
//! it reports the left click, so a click opens the popover and the menu sits on the right click.
//!
//! The icon shows the number of focus items, a dot while a task is active and a cup during a break
//! (docs/KONZEPT.md 4), drawn into the app icon so it works on every desktop. The tooltip and the
//! first menu line show the state: `▶ CORE-418: Webhook-Auth (1/3) · Danach: Release-Notes`.

use std::sync::Mutex;

use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem, ToolTip, TrayMethods};
use tasqlet_core::overview::{Mode, Overview};
use tasqlet_core::{Action, Item};
use tauri::{AppHandle, Emitter, Manager};

use crate::data::{self, Data};
use crate::i18n::{Key, t};
use crate::popover;

/// What the tray shows; it is told again only when this changes.
#[derive(Debug, Clone, PartialEq, Default)]
struct Shown {
    count: usize,
    mode: Option<Mode>,
    status: String,
}

struct TasqletTray {
    app: AppHandle,
    shown: Shown,
    /// The app icon in two sizes, ARGB as the protocol wants.
    icons: Vec<(i32, Vec<u8>)>,
}

#[derive(Default)]
pub struct TrayState {
    handle: Mutex<Option<ksni::Handle<TasqletTray>>>,
    shown: Mutex<Option<Shown>>,
    /// No tray host was found at startup (GNOME without the AppIndicator extension).
    missing: Mutex<bool>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether no tray host was found at startup.
pub fn missing(app: &AppHandle) -> bool {
    app.try_state::<TrayState>()
        .is_some_and(|s| *lock(&s.missing))
}

/// Put the icon into the tray. Without a tray host the popover is shown at once, so Tasqlet is not
/// invisible; a host that starts later (the panel after login) still picks the icon up.
pub fn start(app: &AppHandle) {
    app.manage(TrayState::default());
    let icons = [
        &include_bytes!("../icons/32x32.png")[..],
        &include_bytes!("../icons/64x64.png")[..],
    ]
    .iter()
    .filter_map(|bytes| tauri::image::Image::from_bytes(bytes).ok())
    .map(|img| (img.width() as i32, rgba_to_argb(img.rgba())))
    .collect();
    let tray = TasqletTray {
        app: app.clone(),
        shown: Shown::default(),
        icons,
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if !crate::platform::tray_host_available().await {
            eprintln!("tasqlet: no tray host found – on GNOME install the AppIndicator extension");
            *lock(&app.state::<TrayState>().missing) = true;
            popover::show(&app, None);
        }
        match tray.assume_sni_available(true).spawn().await {
            Ok(handle) => {
                *lock(&app.state::<TrayState>().handle) = Some(handle);
                refresh(&app);
            }
            Err(e) => {
                eprintln!("tasqlet: no tray icon ({e})");
                *lock(&app.state::<TrayState>().missing) = true;
                popover::show(&app, None);
            }
        }
    });
}

/// Show the current state, if it changed.
pub fn refresh(app: &AppHandle) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let Some(handle) = lock(&state.handle).clone() else {
        return;
    };
    let store = app.state::<Data>().store();
    let overview = store.overview();
    let shown = Shown {
        count: overview.focus_count,
        mode: Some(overview.mode),
        status: status_line(&overview),
    };
    {
        let mut last = lock(&state.shown);
        if last.as_ref() == Some(&shown) {
            return;
        }
        *last = Some(shown.clone());
    }
    tauri::async_runtime::spawn(async move {
        handle
            .update(move |t: &mut TasqletTray| t.shown = shown)
            .await;
    });
}

/// The state in one line, for tooltip and menu.
fn status_line(o: &Overview<'_>) -> String {
    let title = |it: &Item, max: usize| short(&it.effective_title(), max);
    let next = o
        .next
        .map(|it| format!(" · {}: {}", t(Key::After), title(it, 30)))
        .unwrap_or_default();
    let progress = o
        .progress
        .map(|(done, all)| format!(" ({done}/{all})"))
        .unwrap_or_default();
    match (o.mode, o.current) {
        (Mode::Working, Some(it)) => format!("▶ {}{progress}{next}", title(it, 40)),
        (Mode::Break, Some(it)) => format!("☕ {} {}", t(Key::BreakThen), title(it, 40)),
        (Mode::Ready, Some(it)) => format!("{}: {}{next}", t(Key::Ready), title(it, 40)),
        _ => t(Key::Idle).to_string(),
    }
}

/// At most `max` characters, with `…` if cut.
fn short(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    }
}

impl ksni::Tray for TasqletTray {
    fn id(&self) -> String {
        "tasqlet".into()
    }

    fn title(&self) -> String {
        "Tasqlet".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        self.icons
            .iter()
            .map(|(size, data)| {
                let mut data = data.clone();
                if self.shown.count > 0 {
                    draw_badge(&mut data, *size, self.shown.count);
                }
                match self.shown.mode {
                    Some(Mode::Working) => draw_dot(&mut data, *size),
                    Some(Mode::Break) => draw_cup(&mut data, *size),
                    _ => {}
                }
                Icon {
                    width: *size,
                    height: *size,
                    data,
                }
            })
            .collect()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: self.shown.status.clone(),
            description: t(Key::ClickHint).into(),
            ..Default::default()
        }
    }

    fn watcher_offline(&self, _reason: ksni::OfflineReason) -> bool {
        // Keep waiting: the panel may start later or restart.
        true
    }

    fn activate(&mut self, x: i32, y: i32) {
        let at = (x != 0 || y != 0).then_some((x, y));
        popover::toggle(&self.app, at);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mode = self.shown.mode;
        let active = matches!(mode, Some(Mode::Working | Mode::Break));
        let (pause_label, pause_action) = if mode == Some(Mode::Break) {
            (t(Key::Resume), Action::Resume)
        } else {
            (t(Key::Pause), Action::Pause)
        };
        vec![
            StandardItem {
                label: self.shown.status.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t(Key::Open).into(),
                activate: Box::new(|t: &mut Self| popover::show(&t.app, None)),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: t(Key::Capture).into(),
                activate: Box::new(|t: &mut Self| {
                    popover::show(&t.app, None);
                    let _ = t.app.emit_to(popover::LABEL, popover::FOCUS_CAPTURE, ());
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: pause_label.into(),
                enabled: active,
                activate: Box::new(move |t: &mut Self| {
                    let _ = data::apply(&t.app, pause_action.clone());
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: t(Key::Finish).into(),
                enabled: active,
                activate: Box::new(|t: &mut Self| {
                    let current = t.app.state::<Data>().store().active().map(|it| it.id);
                    if let Some(id) = current {
                        let _ = data::apply(&t.app, Action::Finish(id));
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t(Key::Quit).into(),
                activate: Box::new(|t: &mut Self| {
                    t.app.state::<Data>().flush();
                    t.app.exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Pixels as ARGB32 in network byte order, as StatusNotifierItem expects.
fn rgba_to_argb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[3], p[0], p[1], p[2]])
        .collect()
}

const WHITE: [u8; 4] = [255, 255, 255, 255];
/// Dark pill behind the number: reads on the orange icon and on light and dark panels.
const INK: [u8; 4] = [255, 29, 27, 24];
/// Blue dot for "in Arbeit", a clear contrast to the orange icon.
const BLUE: [u8; 4] = [255, 47, 129, 247];

fn setter(data: &mut [u8], size: i32) -> impl FnMut(i32, i32, [u8; 4]) + '_ {
    move |x, y, px| {
        if (0..size).contains(&x) && (0..size).contains(&y) {
            let i = ((y * size + x) * 4) as usize;
            data[i..i + 4].copy_from_slice(&px);
        }
    }
}

/// 3×5 pixel digits, one row per entry, bits from the left.
const DIGITS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

/// Pill with the number in the top right corner. 32 px base, scaled for larger icons.
fn draw_badge(data: &mut [u8], size: i32, count: usize) {
    let s = (size / 32).max(1);
    let text = count.min(99).to_string();
    let h = 16 * s;
    let w = ((text.len() as i32 * 8 + 6) * s).max(h);
    let (x0, r) = (size - w, h / 2);
    let mut set = setter(data, size);
    for y in 0..h {
        for x in x0..size {
            // Rounded ends: inside if within the end circles or the straight middle.
            let cx = (x - x0).clamp(r, w - r) + x0;
            if (x - cx).pow(2) + (y - r).pow(2) <= r * r {
                set(x, y, INK);
            }
        }
    }
    let mut cx = x0 + (w - (text.len() as i32 * 8 - 2) * s) / 2;
    for ch in text.chars() {
        let rows = DIGITS[ch.to_digit(10).unwrap_or(0) as usize];
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..3 {
                if bits & (4 >> col) != 0 {
                    for dy in 0..2 * s {
                        for dx in 0..2 * s {
                            set(
                                cx + col * 2 * s + dx,
                                3 * s + row as i32 * 2 * s + dy,
                                WHITE,
                            );
                        }
                    }
                }
            }
        }
        cx += 8 * s;
    }
}

/// Filled circle with a white ring in the bottom right corner; returns its centre and radius.
fn draw_disc(data: &mut [u8], size: i32, outer: i32, fill: [u8; 4]) -> (i32, i32) {
    let s = (size / 32).max(1);
    let (outer, inner) = (outer * s, outer * s - (3 * s) / 2);
    let (cx, cy) = (size - outer - 1, size - outer - 1);
    let mut set = setter(data, size);
    for y in cy - outer..=cy + outer {
        for x in cx - outer..=cx + outer {
            let d = (x - cx).pow(2) + (y - cy).pow(2);
            if d <= outer * outer {
                set(x, y, if d <= inner * inner { fill } else { WHITE });
            }
        }
    }
    (cx, cy)
}

/// Dot for "in Arbeit".
fn draw_dot(data: &mut [u8], size: i32) {
    draw_disc(data, size, 6, BLUE);
}

/// A cup on a white disc for "Pause": body, handle and a line of steam, 8×7 pixels at 32 px.
fn draw_cup(data: &mut [u8], size: i32) {
    const CUP: [&str; 7] = [
        ".#..#...", //
        "........", //
        "######..", //
        "######.#", //
        "######.#", //
        ".####...", //
        "..##....", //
    ];
    let s = (size / 32).max(1);
    let (cx, cy) = draw_disc(data, size, 8, WHITE);
    let (x0, y0) = (cx - 4 * s, cy - 4 * s);
    let mut set = setter(data, size);
    for (row, line) in CUP.iter().enumerate() {
        for (col, c) in line.bytes().enumerate() {
            if c == b'#' {
                for dy in 0..s {
                    for dx in 0..s {
                        set(x0 + col as i32 * s + dx, y0 + row as i32 * s + dy, INK);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(size: i32) -> Vec<u8> {
        vec![0u8; (size * size * 4) as usize]
    }

    fn count(data: &[u8], px: [u8; 4]) -> usize {
        data.as_chunks::<4>().0.iter().filter(|p| **p == px).count()
    }

    #[test]
    fn badge_stays_in_the_top_right_and_draws_digits() {
        for (size, n) in [(32, 3), (32, 42), (64, 7), (32, 1000)] {
            let mut data = blank(size);
            draw_badge(&mut data, size, n);
            assert!(count(&data, WHITE) > 0 && count(&data, INK) > count(&data, WHITE));
            let bottom_left = (((size - 1) * size) * 4) as usize;
            assert_eq!(&data[bottom_left..bottom_left + 4], &[0, 0, 0, 0]);
        }
    }

    #[test]
    fn dot_and_cup_stay_in_the_bottom_right() {
        for size in [32, 64] {
            let mut dot = blank(size);
            draw_dot(&mut dot, size);
            assert!(count(&dot, BLUE) > 10);
            let mut cup = blank(size);
            draw_cup(&mut cup, size);
            assert!(count(&cup, INK) > 10 && count(&cup, WHITE) > count(&cup, INK));
            for data in [&dot, &cup] {
                assert_eq!(&data[0..4], &[0, 0, 0, 0], "top left stays free");
            }
        }
    }

    #[test]
    fn short_titles() {
        assert_eq!(short("kurz", 10), "kurz");
        assert_eq!(
            short("CORE-418: Webhook-Authentifizierung", 16),
            "CORE-418: Webho…"
        );
        assert_eq!(short("a              b", 4), "a…");
    }

    #[test]
    fn argb_order() {
        assert_eq!(rgba_to_argb(&[1, 2, 3, 4]), vec![4, 1, 2, 3]);
    }
}
