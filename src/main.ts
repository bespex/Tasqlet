import { mount } from "svelte";
import App from "./App.svelte";
import Gallery from "./Gallery.svelte";
import "./app.css";
import { getSettings } from "./lib/backend";
import { setLang, systemLang } from "./lib/i18n";
import { DEFAULT_THEME, applyTheme } from "./lib/theme";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app element");

applyTheme(DEFAULT_THEME);

// http://localhost:1430/#bausteine shows the components page (plan 1.5) instead of the popover.
const component = location.hash === "#bausteine" ? Gallery : App;

// The language from the settings counts before anything is drawn; if they cannot be read, the
// system language stays.
const start = () => mount(component, { target });
getSettings()
  .then((s) => setLang(s.language === "auto" ? systemLang() : s.language))
  .catch(() => {})
  .finally(start);
