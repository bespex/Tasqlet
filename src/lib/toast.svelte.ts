// The feedback line at the bottom of the popover (docs/KONZEPT.md 2.4, 7.1): one message at a time,
// inverted so it suits every theme, with "Undo" where the change can be undone.

export interface ToastMessage {
  text: string;
  /** Shown as "Undo" button; the toast closes after it ran. */
  undo?: () => void;
}

/** How long a toast stays: longer when it offers "Undo". */
export const TOAST_MS = { plain: 3500, undo: 6000 };

export const toast = $state<{ current: ToastMessage | null }>({ current: null });

let timer: ReturnType<typeof setTimeout> | undefined;

export function showToast(message: ToastMessage): void {
  toast.current = message;
  clearTimeout(timer);
  timer = setTimeout(hideToast, message.undo ? TOAST_MS.undo : TOAST_MS.plain);
}

export function hideToast(): void {
  clearTimeout(timer);
  toast.current = null;
}
