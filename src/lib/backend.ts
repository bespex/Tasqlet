// The only way the UI talks to the Rust side. Every function has a mock, so the UI runs in a plain
// browser (`npm run dev`) without Tauri. See docs/PLAN.md, "So läuft jeder Schritt".
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { mockSnapshot } from "./mock-data";
import { mockParse } from "./mock-quick";
import type { CaptureKey, Captured, Item, QuickEntry, Snapshot } from "./types";

/** True inside the Tauri webview, false in the browser preview. */
export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** The browser preview's own little store: sample data, changed by capture and undo. */
const preview = {
  snapshot: mockSnapshot(),
  history: [] as Snapshot[],
  listeners: new Set<() => void>(),
  changed() {
    this.listeners.forEach((l) => l());
  },
};

/** Mock values for the browser preview and for tests. */
export const mock = {
  schemaVersion: 1,
  snapshot: () => structuredClone(preview.snapshot),
  /** Back to the sample data (tests). */
  reset() {
    preview.snapshot = mockSnapshot();
    preview.history = [];
  },
};

export async function schemaVersion(): Promise<number> {
  return inTauri() ? invoke<number>("schema_version") : mock.schemaVersion;
}

/** All items plus global state, loaded from the data folder by the core. */
export async function loadSnapshot(): Promise<Snapshot> {
  return inTauri() ? invoke<Snapshot>("snapshot") : mock.snapshot();
}

/** Hides the popover; the app keeps running in the background. No-op in the browser. */
export async function hidePopover(): Promise<void> {
  if (inTauri()) await invoke("hide_popover");
}

/** Calls `onShown` whenever the app has just shown the popover, so the UI can refresh. */
export async function watchShown(onShown: () => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return listen("popover-shown", onShown);
}

/** Calls `onChanged` whenever the items changed (an action, the tray menu, another program). */
export async function watchChanged(onChanged: () => void): Promise<() => void> {
  if (!inTauri()) {
    preview.listeners.add(onChanged);
    return () => preview.listeners.delete(onChanged);
  }
  return listen("store-changed", onChanged);
}

/** Restores the previous version of a damaged data file and leaves read-only mode. */
export async function restoreBackup(): Promise<void> {
  if (inTauri()) await invoke("restore_backup");
}

/** Calls `onCapture` when the tray menu asks for quick capture. */
export async function watchCapture(onCapture: () => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return listen("focus-capture", onCapture);
}

/** What the quick entry recognises in `text` (live while typing). */
export async function parseQuick(text: string): Promise<QuickEntry> {
  return inTauri() ? invoke<QuickEntry>("parse_quick", { text }) : mockParse(text);
}

/** Saves the quick entry with one of the four keys. Rejects if there is nothing to save. */
export async function capture(text: string, key: CaptureKey): Promise<Captured> {
  if (inTauri()) return invoke<Captured>("capture", { text, key });
  const entry = mockParse(text);
  if (!entry.title && entry.resources.length === 0) throw new Error("nothing to save");
  preview.history.push(structuredClone(preview.snapshot));
  const items = preview.snapshot.items;
  const focus = () => items.filter((i) => i.state === "focus").sort((a, b) => a.order - b.order);
  const now = new Date().toISOString();
  const placed: Captured["placed"] = entry.delegate_to ? "delegated" : key;
  const state: Item["state"] = placed === "now" ? "focus" : placed;
  const item: Item = {
    id: `m${Date.now()}`,
    title: entry.title,
    state,
    active: false,
    last_active: false,
    order: 0,
    note: "",
    checklist: [],
    resources: entry.resources.map((r, i) => ({ ...r, id: `mr${Date.now()}${i}`, used_at: null })),
    date: entry.date,
    delegated_to: entry.delegate_to,
    delegated_at: entry.delegate_to ? now : null,
    tags: [],
    created_at: now,
    updated_at: now,
    done_at: null,
  };
  const replaced = placed === "now" && items.some((i) => i.active);
  if (placed === "now") {
    const rest = focus();
    for (const other of items) {
      other.last_active = replaced && other.active;
      other.active = false;
    }
    item.active = true;
    [item, ...rest].forEach((x, i) => (x.order = i));
  } else if (state === "focus" || state === "intermezzo") {
    item.order = items.filter((i) => i.state === state).length;
  }
  items.push(item);
  preview.changed();
  const position = state === "focus" ? focus().findIndex((i) => i.id === item.id) + 1 : null;
  return { title: item.title, placed, position, replaced, delegated_to: entry.delegate_to };
}

/** Undoes the last action. Rejects if there is nothing to undo. */
export async function undo(): Promise<void> {
  if (inTauri()) return invoke("undo");
  const before = preview.history.pop();
  if (!before) throw new Error("nothing to undo");
  preview.snapshot = before;
  preview.changed();
}
