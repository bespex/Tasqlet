// Data model as seen by the UI. Mirrors docs/KONZEPT.md 2.2; the Rust types in tasqlet-core
// (plan step 1.1) are the source of truth and serialize to exactly these shapes.

export type State = "idea" | "focus" | "intermezzo" | "delegated" | "done";
export type ResourceType = "url" | "file" | "mail";

export interface Resource {
  id: string;
  type: ResourceType;
  target: string;
  /** null = smart label */
  label: string | null;
  used_at: string | null;
  /** What the chip shows: the own label or the smart label, computed by the core (snapshot only). */
  shown?: string;
  /** The fixed `Alt+n` digit (order of adding, 1 to 9); the list comes most important first. */
  digit?: number | null;
}

export interface ChecklistItem {
  id: string;
  text: string;
  done: boolean;
}

export interface Item {
  id: string;
  title: string;
  state: State;
  active: boolean;
  last_active: boolean;
  order: number;
  note: string;
  checklist: ChecklistItem[];
  resources: Resource[];
  /** only noted, never evaluated (D2) */
  date: string | null;
  delegated_to: string | null;
  delegated_at: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
  done_at: string | null;
}

export interface Snapshot {
  schema: number;
  on_break: boolean;
  items: Item[];
  /** The data could not be loaded; the app runs read-only. */
  load_error?: LoadError | null;
  /** No tray host was found at startup (GNOME without the AppIndicator extension). */
  tray_missing?: boolean;
}

export interface LoadError {
  message: string;
  /** A previous version exists and can be restored (`restoreBackup`). */
  backup: boolean;
}

/** What the quick entry recognised while typing (core: `quick::QuickEntry`). */
export interface QuickEntry {
  title: string;
  /** Noted date, "YYYY-MM-DD" */
  date: string | null;
  delegate_to: string | null;
  resources: { type: ResourceType; target: string; label: string | null; shown: string }[];
}

/** The key the quick entry was saved with (docs/KONZEPT.md 3.1). */
export type CaptureKey = "now" | "focus" | "intermezzo" | "idea";

/** What was saved, for the feedback line. */
export interface Captured {
  title: string;
  placed: CaptureKey | "delegated";
  /** Position in the focus list (1-based) for "now" and "focus" */
  position: number | null;
  /** "now": another task was active and moved to #2 */
  replaced: boolean;
  delegated_to: string | null;
}

/** A change to one item (core: `Action`; shell: `act::Act`). */
export type Act =
  | { kind: "start"; id: string }
  | { kind: "finish"; id: string }
  | { kind: "pause" }
  | { kind: "resume" }
  | { kind: "delegate"; id: string; to: string }
  | { kind: "send_to"; id: string; to: "focus" | "intermezzo" | "idea" }
  | { kind: "to_back"; id: string }
  | { kind: "to_next"; id: string }
  | { kind: "move_by"; id: string; delta: number }
  | { kind: "rename"; id: string; title: string }
  | { kind: "set_note"; id: string; note: string }
  | { kind: "add_check"; id: string; text: string }
  | { kind: "toggle_check"; id: string; check: string }
  | { kind: "add_resource"; id: string; target: string }
  | { kind: "brainstorm"; id: string; text: string }
  | { kind: "rename_check"; id: string; check: string; text: string }
  | { kind: "remove_check"; id: string; check: string }
  | { kind: "set_date"; id: string; date: string | null }
  | { kind: "remove_resource"; id: string; resource: string };

/** What the search returns: hits best first, or the recently finished items without a query. */
export interface SearchResult {
  recent: boolean;
  items: Item[];
}

export type Language = "auto" | "de" | "en";
export type PopoverPosition = "corner" | "center";

export type ToggleKeyProblem = "invalid" | "taken" | "unsupported";

/** What the settings dialog shows (shell: `prefs::get_settings`). */
export interface Settings {
  language: Language;
  popover_position: PopoverPosition;
  /** Colour theme and style (`@bespex/themes`); missing until the user chooses (plan 2.6). */
  appearance?: unknown;
  autostart: boolean;
  /** No icon position is known (Wayland): the position setting matters. */
  wayland: boolean;
  /** Global key that shows or hides the popover (`Ctrl+Alt+KeyT`), empty for none (D36). */
  toggle_key: string;
  /** Why that key is not held. */
  toggle_key_problem: ToggleKeyProblem | null;
  /** Learned Jira projects with their base URL (D26). */
  jira: { project: string; base: string }[];
}
