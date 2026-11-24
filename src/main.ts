import { mount } from "svelte";
import App from "./App.svelte";
import Gallery from "./Gallery.svelte";
import "./app.css";
import { DEFAULT_THEME, applyTheme } from "./lib/theme";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app element");

applyTheme(DEFAULT_THEME);

// http://localhost:1430/#bausteine shows the components page (plan 1.5) instead of the popover.
const component = location.hash === "#bausteine" ? Gallery : App;
export default mount(component, { target });
