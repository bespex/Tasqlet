// Sample data for the browser preview and for tests, taken from the click dummy
// (docs/click-dummy/index.html). Dates are relative to "now" so the preview always reads naturally.
import type { ChecklistItem, Item, Resource, Snapshot, State } from "./types";

const DAY = 86_400_000;

function isoDay(offsetDays: number, now: Date): string {
  const d = new Date(now.getFullYear(), now.getMonth(), now.getDate() + offsetDays);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

interface Seed {
  title: string;
  state?: State;
  active?: boolean;
  order?: number;
  note?: string;
  checklist?: [string, boolean?][];
  resources?: [Resource["type"], string, string][];
  date?: number;
  delegated_to?: string;
  delegated_days_ago?: number;
  created_hours_ago?: number;
  done_days_ago?: number;
}

const seeds: Seed[] = [
  {
    title: "CORE-418: Webhook-Authentifizierung umstellen",
    state: "focus",
    active: true,
    order: 0,
    date: 0,
    resources: [
      ["url", "https://jira.example.com/browse/CORE-418", "CORE-418"],
      ["mail", "~/Mail/ops-webhook.eml", "Mail: Ops"],
      ["file", "~/projekte/core/docs/spec.md", "spec.md"],
      ["url", "https://github.com/example/core/pull/57", "PR #57"],
      ["file", "~/projekte/core/gateway/", "gateway"],
    ],
    checklist: [
      ["Alten Token-Flow im Staging prüfen", true],
      ["Header-Validierung im Gateway anpassen"],
      ["Deployment-Fenster abstimmen"],
    ],
    note: "Staging nutzt noch X-Legacy. Bei Rückfragen an Sarah wenden (ab 14 Uhr erreichbar).",
  },
  {
    title: "Release-Notes für v2.4 finalisieren",
    state: "focus",
    order: 1,
    checklist: [
      ["Changelog aus Git ziehen", true],
      ["Breaking Changes markieren", true],
      ["Upgrade-Hinweise schreiben"],
      ["Review durch Produkt"],
    ],
  },
  { title: "Angebot für Kunde X gegenlesen", state: "focus", order: 2, date: 3 },
  {
    title: "Memory-Leak im Worker-Pool prüfen",
    state: "focus",
    order: 3,
    resources: [["url", "https://github.com/example/worker/issues/212", "worker#212"]],
    note: "Heap wächst nur unter Last über 200 req/s.",
  },
  { title: "TLS-Zertifikat für Staging verlängern", state: "focus", order: 4 },
  { title: "VPN-Token erneuern", state: "intermezzo", order: 0 },
  { title: "Paketschein drucken", state: "intermezzo", order: 1 },
  { title: "Rückruf Hausverwaltung", state: "intermezzo", order: 2 },
  { title: "Kaffeemaschine entkalken", state: "intermezzo", order: 3 },
  {
    title: "CLI-Tool für Log-Analyse bauen",
    created_hours_ago: 2,
    note: "- evtl. in Rust statt Python wegen Startzeit\n- Filter nach Trace-ID muss per Pipe funktionieren",
    resources: [["url", "https://github.com/BurntSushi/ripgrep", "Vorbild: ripgrep"]],
  },
  { title: "Onboarding-Doku für neue Devs vereinfachen", created_hours_ago: 26 },
  {
    title: "Dark-Mode für Maillet",
    created_hours_ago: 72,
    note: "- Farben aus dem System-Theme übernehmen\n- Kontrast der Code-Karte prüfen\n- Umschalter in den Einstellungen?\n- Screenshots fürs README neu machen",
    checklist: [["Farbrollen sammeln"]],
  },
  { title: "Backup-Skript prüfen", created_hours_ago: 144, note: "- läuft seit Oktober nicht mehr?" },
  { title: "Tasqlet mit HatRack koppeln: Filter nach Hut", created_hours_ago: 216 },
  {
    title: "Load-Test-Umgebung aufsetzen",
    state: "delegated",
    delegated_to: "Jonas",
    delegated_days_ago: 3,
    date: 2,
    checklist: [["k6-Skripte übernehmen", true], ["Grafana-Board anlegen"]],
    note: "Braucht Zugriff auf den Staging-Cluster.",
  },
  {
    title: "Rechnung Q3 klären",
    state: "delegated",
    delegated_to: "Buchhaltung",
    delegated_days_ago: 6,
    resources: [["mail", "~/Mail/rechnung-0931.eml", "Mail: Rechnung 0931"]],
  },
  {
    title: "Nginx-Logrotation korrigieren",
    state: "done",
    done_days_ago: 1,
    note: "copytruncate statt create, sonst schreibt der Worker ins Leere.",
  },
  {
    title: "Folien für Architektur-Review",
    state: "done",
    done_days_ago: 5,
    resources: [["file", "~/Dokumente/arch-review.odp", "arch-review.odp"]],
  },
];

/** The sample texts in English, for the English preview and the screenshots of the website. */
const english: Record<string, string> = {
  "CORE-418: Webhook-Authentifizierung umstellen": "CORE-418: Switch webhook authentication",
  "Alten Token-Flow im Staging prüfen": "Check the old token flow on staging",
  "Header-Validierung im Gateway anpassen": "Adapt header validation in the gateway",
  "Deployment-Fenster abstimmen": "Agree on the deployment window",
  "Staging nutzt noch X-Legacy. Bei Rückfragen an Sarah wenden (ab 14 Uhr erreichbar).":
    "Staging still uses X-Legacy. Questions go to Sarah (available from 2 pm).",
  "Release-Notes für v2.4 finalisieren": "Finish the release notes for v2.4",
  "Changelog aus Git ziehen": "Pull the changelog from Git",
  "Breaking Changes markieren": "Mark breaking changes",
  "Upgrade-Hinweise schreiben": "Write upgrade notes",
  "Review durch Produkt": "Review by product",
  "Angebot für Kunde X gegenlesen": "Proofread the offer for customer X",
  "Memory-Leak im Worker-Pool prüfen": "Look into the memory leak in the worker pool",
  "Heap wächst nur unter Last über 200 req/s.": "The heap only grows under load above 200 req/s.",
  "TLS-Zertifikat für Staging verlängern": "Renew the TLS certificate for staging",
  "VPN-Token erneuern": "Renew the VPN token",
  "Paketschein drucken": "Print the parcel label",
  "Rückruf Hausverwaltung": "Call back the landlord",
  "Kaffeemaschine entkalken": "Descale the coffee machine",
  "CLI-Tool für Log-Analyse bauen": "Build a CLI tool for log analysis",
  "- evtl. in Rust statt Python wegen Startzeit\n- Filter nach Trace-ID muss per Pipe funktionieren":
    "- maybe Rust instead of Python, for the start-up time\n- filtering by trace ID has to work in a pipe",
  "Vorbild: ripgrep": "Model: ripgrep",
  "Onboarding-Doku für neue Devs vereinfachen": "Simplify the onboarding docs for new devs",
  "Dark-Mode für Maillet": "Dark mode for Maillet",
  "- Farben aus dem System-Theme übernehmen\n- Kontrast der Code-Karte prüfen\n- Umschalter in den Einstellungen?\n- Screenshots fürs README neu machen":
    "- take the colours from the system theme\n- check the contrast of the code card\n- a switch in the settings?\n- new screenshots for the README",
  "Farbrollen sammeln": "Collect the colour roles",
  "Backup-Skript prüfen": "Check the backup script",
  "- läuft seit Oktober nicht mehr?": "- not running since October?",
  "Tasqlet mit HatRack koppeln: Filter nach Hut": "Couple Tasqlet with HatRack: filter by hat",
  "Load-Test-Umgebung aufsetzen": "Set up the load test environment",
  "k6-Skripte übernehmen": "Take over the k6 scripts",
  "Grafana-Board anlegen": "Create a Grafana board",
  "Braucht Zugriff auf den Staging-Cluster.": "Needs access to the staging cluster.",
  "Rechnung Q3 klären": "Sort out the Q3 invoice",
  Buchhaltung: "Accounting",
  "Mail: Rechnung 0931": "Mail: Invoice 0931",
  "~/Mail/rechnung-0931.eml": "~/Mail/invoice-0931.eml",
  "~/projekte/core/docs/spec.md": "~/projects/core/docs/spec.md",
  "~/projekte/core/gateway/": "~/projects/core/gateway/",
  "Nginx-Logrotation korrigieren": "Fix the nginx log rotation",
  "copytruncate statt create, sonst schreibt der Worker ins Leere.":
    "copytruncate instead of create, or the worker writes into the void.",
  "Folien für Architektur-Review": "Slides for the architecture review",
  "~/Dokumente/arch-review.odp": "~/Documents/arch-review.odp",
};

/** Builds the sample snapshot, in German or English. Deterministic for a given `now`. */
export function mockSnapshot(now: Date = new Date(), lang: "de" | "en" = "de"): Snapshot {
  const tr = (text: string) => (lang === "en" ? (english[text] ?? text) : text);
  let n = 0;
  const id = (p: string) => `${p}${++n}`;
  const at = (msAgo: number) => new Date(now.getTime() - msAgo).toISOString();
  const items: Item[] = seeds.map((s) => {
    const checklist: ChecklistItem[] = (s.checklist ?? []).map(([text, done]) => ({
      id: id("c"),
      text: tr(text),
      done: !!done,
    }));
    // The third value is what the chip shows; only "Mail: …" for a .eml is an own label here,
    // the rest are smart labels the core would compute.
    const resources: Resource[] = (s.resources ?? []).map(([type, target, shown]) => ({
      id: id("r"),
      type,
      target: tr(target),
      label: type === "mail" ? tr(shown) : null,
      used_at: null,
      shown: tr(shown),
    }));
    const created = at((s.created_hours_ago ?? 48) * 3_600_000);
    return {
      id: id("i"),
      title: tr(s.title),
      state: s.state ?? "idea",
      active: !!s.active,
      last_active: false,
      order: s.order ?? 0,
      note: tr(s.note ?? ""),
      checklist,
      resources,
      date: s.date === undefined ? null : isoDay(s.date, now),
      delegated_to: s.delegated_to === undefined ? null : tr(s.delegated_to),
      delegated_at: s.delegated_days_ago === undefined ? null : at(s.delegated_days_ago * DAY),
      tags: [],
      created_at: created,
      updated_at: created,
      done_at: s.done_days_ago === undefined ? null : at(s.done_days_ago * DAY),
    };
  });
  return { schema: 1, on_break: false, items };
}
