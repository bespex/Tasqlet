import { mount, tick } from "svelte";
import App from "./App.svelte";
import Gallery from "./Gallery.svelte";
import SettingsView from "./lib/ui/SettingsView.svelte";
import "./app.css";
import { getSettings, windowReady } from "./lib/backend";
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
// the system language and the default appearance stay. The settings window is built hidden and
// shown only once it is drawn with its data (`windowReady`), so it does not visibly build up.
const settings = getSettings()
  .then((s) => {
    setLang(s.language === "auto" ? systemLang() : s.language);
    applyAppearance(appearanceOf(s.appearance));
    return s;
  })
  .catch(() => undefined);

if (settingsWindow) {
  void settings.then(async (initial) => {
    mount(SettingsView, { target, props: { initial } });
    await tick();
    requestAnimationFrame(() => void windowReady().catch(() => {}));
  });
} else {
  void settings.then(() => mount(page, { target }));
}
