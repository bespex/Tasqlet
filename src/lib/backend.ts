// The only way the UI talks to the Rust side. Every function has a mock, so the UI runs in a plain
// browser (`npm run dev`) without Tauri. See docs/PLAN.md, "So läuft jeder Schritt".
import { invoke } from "@tauri-apps/api/core";

/** True inside the Tauri webview, false in the browser preview. */
export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Mock values for the browser preview and for tests. */
export const mock = {
  schemaVersion: 1,
};

export async function schemaVersion(): Promise<number> {
  return inTauri() ? invoke<number>("schema_version") : mock.schemaVersion;
}
