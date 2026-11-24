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
