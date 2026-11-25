// The only way the UI talks to the Rust side. Every function has a mock, so the UI runs in a plain
// browser (`npm run dev`) without Tauri. See docs/PLAN.md, "So läuft jeder Schritt".
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { mockSnapshot } from "./mock-data";
import { mockParse } from "./mock-quick";
import { mockRecentlyDone, mockSearch, searchTerms } from "./mock-search";
import type {
  Act,
  CaptureKey,
  Captured,
  Item,
  Language,
  PopoverPosition,
  QuickEntry,
  SearchResult,
  Settings,
  Snapshot,
} from "./types";

/** True inside the Tauri webview, false in the browser preview. */
export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Ids of the preview's own items: a counter, not the clock, so two in one millisecond differ. */
let lastId = 0;
const newId = (prefix: string) => `${prefix}${Date.now()}-${++lastId}`;

/** The browser preview's own little store: sample data, changed by capture and undo. */
const preview = {
  snapshot: mockSnapshot(),
  history: [] as Snapshot[],
  listeners: new Set<() => void>(),
  changed() {
    this.listeners.forEach((l) => l());
  },
};

/** Most important resource first (last opened, then order of adding), each with its fixed digit. */
export function rankResources(items: Item[]): Item[] {
  for (const item of items) {
    const added = item.resources.map((r, index) => ({ ...r, digit: index < 9 ? index + 1 : null, index }));
    added.sort((a, b) => (b.used_at ?? "").localeCompare(a.used_at ?? "") || a.index - b.index);
    item.resources = added.map(({ index: _index, ...r }) => r);
  }
  return items;
}

/** Mock values for the browser preview and for tests. */
export const mock = {
  schemaVersion: 1,
  snapshot: () => {
    const copy = structuredClone(preview.snapshot);
    rankResources(copy.items);
    return copy;
  },
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

/** Calls `onHidden` whenever the popover was hidden (focus lost, Esc, tray), so the UI can step back. */
export async function watchHidden(onHidden: () => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return listen("popover-hidden", onHidden);
}

/** Makes the popover twice as wide for the elaboration, or normal again. No-op in the browser. */
export async function setWide(wide: boolean): Promise<void> {
  if (inTauri()) await invoke("set_wide", { wide });
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
    id: newId("m"),
    title: entry.title,
    state,
    active: false,
    last_active: false,
    order: 0,
    note: "",
    checklist: [],
    resources: entry.resources.map((r, i) => ({ ...r, id: newId("mr"), used_at: null })),
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

/** Closes the gaps in the order of focus and intermezzo, the active item first (preview only). */
function renumber(items: Item[]): void {
  for (const state of ["focus", "intermezzo"] as const) {
    items
      .filter((i) => i.state === state)
      .sort((a, b) => Number(b.active) - Number(a.active) || a.order - b.order)
      .forEach((i, n) => (i.order = n));
  }
}

/** Carries out a change to one item. Rejects if the core would refuse it. */
export async function act(change: Act): Promise<void> {
  if (inTauri()) return invoke("act", { act: change });
  const snap = preview.snapshot;
  const items = snap.items;
  const find = (id: string) => {
    const item = items.find((i) => i.id === id);
    if (!item) throw new Error("unknown item");
    return item;
  };
  const edit = change.kind === "set_note";
  const before = structuredClone(snap);
  const now = new Date().toISOString();
  const move = (item: Item, state: Item["state"]) => {
    item.state = state;
    item.active = false;
    item.last_active = false;
    item.order = 9999;
    item.updated_at = now;
  };
  switch (change.kind) {
    case "start": {
      const item = find(change.id);
      const left = items.find((i) => i.active && i.id !== item.id);
      items.forEach((i) => ((i.last_active = i === left), (i.active = false)));
      item.state = "focus";
      item.active = true;
      item.last_active = false;
      item.order = -1;
      snap.on_break = false;
      break;
    }
    case "finish": {
      const item = find(change.id);
      move(item, "done");
      item.done_at = now;
      break;
    }
    case "pause":
      if (!items.some((i) => i.active)) throw new Error("no active item");
      snap.on_break = true;
      break;
    case "resume":
      snap.on_break = false;
      break;
    case "delegate": {
      const to = change.to.trim().replace(/^@/, "");
      if (!to) throw new Error("empty name");
      const item = find(change.id);
      move(item, "delegated");
      item.delegated_to = to;
      item.delegated_at = now;
      break;
    }
    case "send_to":
      move(find(change.id), change.to);
      break;
    case "to_back": {
      const item = find(change.id);
      const last = Math.max(...items.filter((i) => i.state === "focus").map((i) => i.order));
      if (item.state !== "focus" || item.order === last) throw new Error("not applicable");
      snap.on_break = snap.on_break && !item.active;
      item.active = false;
      item.last_active = false;
      item.order = 9999;
      break;
    }
    case "to_next": {
      const item = find(change.id);
      if (item.state === "focus" && item.order === 0) throw new Error("not applicable");
      if (item.state === "focus" && item.order === 1) return;
      const queue = items.filter((i) => i.state === "focus" && i !== item).sort((a, b) => a.order - b.order);
      item.state = "focus";
      item.active = false;
      item.last_active = false;
      queue.splice(Math.min(1, queue.length), 0, item);
      queue.forEach((i, n) => (i.order = n));
      break;
    }
    case "move_by": {
      const item = find(change.id);
      const queue = items.filter((i) => i.state === "focus").sort((a, b) => a.order - b.order);
      const from = queue.indexOf(item);
      const to = from + change.delta;
      if (item.state !== "focus" || from < 1 || to < 1 || to >= queue.length || change.delta === 0)
        throw new Error("not applicable");
      queue.splice(to, 0, queue.splice(from, 1)[0]);
      queue.forEach((i, n) => (i.order = n));
      break;
    }
    case "brainstorm": {
      const text = change.text.trim();
      if (!text) throw new Error("empty name");
      const item = find(change.id);
      const found = mockParse(text).resources[0];
      if (found && found.target === text) {
        if (!item.resources.some((r) => r.target === text))
          item.resources.push({ ...found, id: newId("mr"), used_at: null });
      } else {
        item.note = `${item.note}${item.note && !item.note.endsWith("\n") ? "\n" : ""}- ${text}`;
      }
      break;
    }
    case "remove_resource": {
      const item = find(change.id);
      const before = item.resources.length;
      item.resources = item.resources.filter((r) => r.id !== change.resource);
      if (item.resources.length === before) throw new Error("unknown resource");
      break;
    }
    case "rename": {
      const title = change.title.trim();
      const item = find(change.id);
      if (!title && item.state !== "idea") throw new Error("empty name");
      item.title = title;
      break;
    }
    case "set_note":
      find(change.id).note = change.note;
      break;
    case "add_check": {
      const text = change.text.trim();
      if (!text) throw new Error("empty name");
      find(change.id).checklist.push({ id: newId("mc"), text, done: false });
      break;
    }
    case "toggle_check": {
      const step = find(change.id).checklist.find((c) => c.id === change.check);
      if (!step) throw new Error("unknown step");
      step.done = !step.done;
      break;
    }
    case "rename_check": {
      const text = change.text.trim();
      if (!text) throw new Error("empty name");
      const step = find(change.id).checklist.find((c) => c.id === change.check);
      if (!step) throw new Error("unknown step");
      step.text = text;
      break;
    }
    case "remove_check": {
      const item = find(change.id);
      if (!item.checklist.some((c) => c.id === change.check)) throw new Error("unknown step");
      item.checklist = item.checklist.filter((c) => c.id !== change.check);
      break;
    }
    case "set_date":
      find(change.id).date = change.date;
      break;
    case "add_resource": {
      const target = change.target.trim();
      const entry = mockParse(target);
      const found = entry.resources[0];
      if (!found || found.target !== target) throw new Error("not a resource");
      const item = find(change.id);
      if (item.resources.some((r) => r.target === target)) return;
      item.resources.push({ ...found, id: newId("mr"), used_at: null });
      break;
    }
  }
  renumber(items);
  if (!edit) preview.history.push(before);
  preview.changed();
}

/** Opens one resource of an item and ranks it first. The preview only marks it as used. */
export async function openResource(id: string, resource: string): Promise<void> {
  if (inTauri()) return invoke("open_resource", { id, resource });
  const found = preview.snapshot.items.find((i) => i.id === id)?.resources.find((r) => r.id === resource);
  if (!found) throw new Error("unknown resource");
  found.used_at = new Date().toISOString();
  preview.changed();
}

/** Opens all resources of an item, most important first. */
export async function openAll(id: string): Promise<void> {
  if (inTauri()) return invoke("open_all", { id });
  const item = preview.snapshot.items.find((i) => i.id === id);
  if (!item) throw new Error("unknown item");
  const now = new Date().toISOString();
  item.resources.forEach((r) => (r.used_at = now));
  preview.changed();
}

/** Full text search over everything, finished items included; without words the recently finished. */
export async function search(query: string): Promise<SearchResult> {
  if (inTauri()) return invoke<SearchResult>("search", { query });
  const items = mock.snapshot().items;
  const recent = searchTerms(query).length === 0;
  return { recent, items: recent ? mockRecentlyDone(items) : mockSearch(items, query) };
}

/** The preview's settings live in the session, so a reload (language switch) keeps them. */
let previewMemory: Settings | null = null;
const previewSettings = (): Settings => {
  const saved = typeof sessionStorage !== "undefined" ? sessionStorage.getItem("tq-settings") : null;
  if (saved) return JSON.parse(saved) as Settings;
  return (
    previewMemory ?? {
      language: "auto",
      popover_position: "corner",
      autostart: false,
      wayland: false,
      toggle_key: "",
      toggle_key_problem: null,
      jira: [{ project: "CORE", base: "https://jira.example.com" }],
    }
  );
};
const keepPreview = (s: Settings) => {
  previewMemory = s;
  if (typeof sessionStorage !== "undefined") sessionStorage.setItem("tq-settings", JSON.stringify(s));
};

/** Settings for the dialog: language, popover position, autostart, learned Jira projects. */
export async function getSettings(): Promise<Settings> {
  return inTauri() ? invoke<Settings>("get_settings") : previewSettings();
}

/** Saves language and popover position. A new language shows after a reload (and in the tray after a restart). */
export async function setSettings(language: Language, popoverPosition: PopoverPosition): Promise<void> {
  if (inTauri()) return invoke("set_settings", { language, popoverPosition });
  keepPreview({ ...previewSettings(), language, popover_position: popoverPosition });
}

/** Switches autostart at login on or off. */
export async function setAutostart(enabled: boolean): Promise<void> {
  if (inTauri()) return invoke("set_autostart", { enabled });
  keepPreview({ ...previewSettings(), autostart: enabled });
}

/** A new global key for the popover (empty: none). Rejects with "invalid", "taken" or "unsupported";
 *  then the old key stays (D36). */
export async function setToggleKey(keys: string): Promise<void> {
  if (inTauri()) return invoke("set_toggle_key", { keys });
  if (keys === "Ctrl+Alt+KeyX") throw "taken"; // to see the message in the preview
  keepPreview({ ...previewSettings(), toggle_key: keys, toggle_key_problem: null });
}

/** While a key is recorded, the global one is let go, so pressing it does not hide the popover. */
export async function suspendToggleKey(suspended: boolean): Promise<void> {
  if (inTauri()) return invoke("suspend_toggle_key", { suspended });
}

/** Forgets a learned Jira project (D26). */
export async function forgetJira(project: string): Promise<void> {
  if (inTauri()) return invoke("forget_jira", { project });
  const s = previewSettings();
  keepPreview({ ...s, jira: s.jira.filter((j) => j.project !== project) });
}
