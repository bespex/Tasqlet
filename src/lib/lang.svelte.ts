// A counter the texts read: when the language changes, everything that drew a text with `t()` in a
// component or a `$derived` draws it again, so no reload is needed (the settings stay open).
export const langRevision = $state({ n: 0 });
