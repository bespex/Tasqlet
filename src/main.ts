import { mount } from "svelte";
import App from "./App.svelte";
import Gallery from "./Gallery.svelte";
import SettingsView from "./lib/ui/SettingsView.svelte";
import "./app.css";
import { getSettings } from "./lib/backend";
import { setLang, systemLang } from "./lib/i18n";
import { DEFAULT_APPEARANCE, appearanceOf, applyAppearance } from "./lib/theme";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app element");

applyAppearance(DEFAULT_APPEARANCE);

// http://localhost:1430/#bausteine shows the components page (plan 1.5) instead of the popover;
// ?view=settings is the settings window (D40).
const settingsWindow = new URLSearchParams(location.search).get("view") === "settings";
const page = location.hash === "#bausteine" ? Gallery : App;

// Language and appearance from the settings count before anything is drawn; if they cannot be read,
// the system language and the default appearance stay.
const start = () => (settingsWindow ? mount(SettingsView, { target, props: {} }) : mount(page, { target }));
getSettings()
  .then((s) => {
    setLang(s.language === "auto" ? systemLang() : s.language);
    applyAppearance(appearanceOf(s.appearance));
  })
  .catch(() => {})
  .finally(start);
