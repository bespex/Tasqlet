// A rough stand-in for the core's quick-entry parser (crates/tasqlet-core/src/quick.rs), only for
// the browser preview. The app always parses in Rust; this needs to be good enough to try the UI.
import type { QuickEntry, ResourceType } from "./types";

const jiraProjects = new Map<string, string>([["CORE", "https://jira.example.com"]]);

function iso(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function mockDate(word: string, today: Date): string | null {
  const w = word.toLowerCase();
  const plus = (n: number) => iso(new Date(today.getFullYear(), today.getMonth(), today.getDate() + n));
  if (w === "heute" || w === "today") return plus(0);
  if (w === "morgen" || w === "tomorrow") return plus(1);
  if (w === "übermorgen") return plus(2);
  const days = ["so", "mo", "di", "mi", "do", "fr", "sa"].indexOf(w.slice(0, 2));
  if (days >= 0 && /^[a-zäöü]+$/.test(w)) return plus((days - today.getDay() + 7) % 7);
  const m = w.match(/^(\d{1,2})\.(\d{1,2})\.?$/);
  if (m) {
    const d = new Date(today.getFullYear(), +m[2] - 1, +m[1]);
    if (d < new Date(today.getFullYear(), today.getMonth(), today.getDate())) d.setFullYear(d.getFullYear() + 1);
    return iso(d);
  }
  return null;
}

function label(target: string): string {
  const jira = target.match(/\/browse\/([A-Z][A-Z0-9]+-\d+)/);
  if (jira) return jira[1];
  const pr = target.match(/github\.com\/[^/]+\/[^/]+\/pull\/(\d+)/);
  if (pr) return `PR #${pr[1]}`;
  if (/^https?:\/\//.test(target)) return new URL(target).hostname.replace(/^www\./, "");
  return target.replace(/\/$/, "").split("/").pop() ?? target;
}

function kind(word: string): ResourceType | null {
  if (/^https?:\/\/\S+\.\S+/.test(word)) return "url";
  if (/^(mailto:|mid:)|\.eml$/i.test(word)) return "mail";
  if (/^(~\/|\/|file:\/\/)\S/.test(word)) return "file";
  return null;
}

export function mockParse(text: string, today: Date = new Date()): QuickEntry {
  const entry: QuickEntry = { title: "", date: null, delegate_to: null, resources: [] };
  const words: string[] = [];
  for (const raw of text.split(/\s+/).filter(Boolean)) {
    const word = raw.replace(/^[<("]+|[.,;:!?)>"]+$/g, "");
    const jiraLink = word.match(/^(https?:\/\/.+)\/browse\/([A-Z][A-Z0-9]+)-\d+/);
    if (jiraLink) jiraProjects.set(jiraLink[2], jiraLink[1]);
    if (raw.startsWith("@") && !entry.delegate_to && raw.length > 1) {
      entry.delegate_to = raw.slice(1).replace(/[^\p{L}\d._-].*$/u, "");
      continue;
    }
    if (raw.startsWith("!") && !entry.date) {
      const date = mockDate(raw.slice(1), today);
      if (date) {
        entry.date = date;
        continue;
      }
    }
    const type = kind(word);
    if (type) {
      entry.resources.push({ type, target: word, label: null, shown: label(word) });
      continue;
    }
    const key = word.match(/^([A-Z][A-Z0-9]+)-\d+$/);
    const base = key && jiraProjects.get(key[1]);
    if (base) entry.resources.push({ type: "url", target: `${base}/browse/${word}`, label: null, shown: word });
    words.push(raw);
  }
  entry.title = words.join(" ") || entry.resources[0]?.shown || "";
  return entry;
}
