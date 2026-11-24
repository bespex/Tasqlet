// The only way the UI talks to the Rust side. Every function has a mock, so the UI runs in a plain
// browser (`npm run dev`) without Tauri. See docs/PLAN.md, "So läuft jeder Schritt".
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { mockSnapshot } from "./mock-data";
import type { Snapshot } from "./types";

/** True inside the Tauri webview, false in the browser preview. */
export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Mock values for the browser preview and for tests. */
export const mock = {
  schemaVersion: 1,
  snapshot: mockSnapshot,
};

export async function schemaVersion(): Promise<number> {
  return inTauri() ? invoke<number>("schema_version") : mock.schemaVersion;
}

/** All open items plus global state. Until storage exists (plan 1.2) the app returns an empty list. */
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
