# Tasqlet: Umsetzungsplan

Stand 2026-11-24. Vorgabe: `docs/KONZEPT.md`. Zielbild: `docs/click-dummy/`.

## Einstieg für eine neue Session

1. Diesen Abschnitt und „So läuft jeder Schritt“ lesen.
2. In der Übersicht unten den ersten offenen Schritt suchen (`[ ]`). Nur diesen bearbeiten.
3. Aus `KONZEPT.md` nur die Abschnitte lesen, die der Schritt nennt. Den Dummy öffnen, wenn der Schritt
   Oberfläche betrifft.
4. Vorlagen aus den Geschwisterprojekten nutzen, statt neu zu erfinden. Pfade stehen beim jeweiligen
   Schritt: `../Maillet` (Tray mit `ksni`, Popover-Positionierung, Packaging), `../CodeDeck`
   (Einzelinstanz-Socket, `--toggle`, Browser-Vorschau mit Mock, Theme-Prüfung, i18n).

## So läuft jeder Schritt

1. **Klären.** Ist etwas im Konzept offen oder widerspricht der Dummy dem Konzept: kurz nachfragen,
   bevor gebaut wird. Abweichungen vom Dummy nur nach Rückfrage.
2. **Core zuerst.** Domänenlogik kommt in `tasqlet-core`, mit Tests, ohne GUI. Die Funktionen im Dummy
   (`start`, `finish`, `toNext`, `moveBy`, `setState`, `delegate`) zeigen das erwartete Verhalten.
3. **Dann die Oberfläche.** Sie bekommt Daten nur über `src/backend.ts`. Jede Funktion dort hat einen
   Mock, damit die Ansicht in der Browser-Vorschau (`npm run dev`) ohne Tauri läuft (wie CodeDeck).
4. **Prüfen.** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `npm run check`,
   `npm test`. Bei Oberfläche zusätzlich ein Screenshot der Vorschau und der Vergleich mit dem Dummy.
5. **Abschließen.** Abnahme des Schritts vorführen. Den Haken hier setzen. Neue Erkenntnisse als
   Entscheidung in `docs/ENTSCHEIDUNGEN.md` festhalten, bei Bedarf `KONZEPT.md` anpassen.

## Entscheidungen vor dem Start

| # | Frage | Vorschlag | Stand |
|---|---|---|---|
| E1 | Frontend: Svelte 5 oder React | Svelte 5 | **entschieden: Svelte 5** (D12) |
| E2 | Ziffer `Alt+n` fest am Arbeitsmittel oder an der Anzeige | fest (`KONZEPT.md` 5) | **entschieden: fest** (D6) |
| E3 | Sprache von Code, Kommentaren, Doku | Code und Kommentare Englisch, Doku Deutsch | **entschieden, vorerst** (D13) |
| E4 | Commit-Regeln | Commits bündeln, wo sinnvoll | **entschieden** (D14), steht in `CLAUDE.md` |

## Übersicht

- **Phase 0: Gerüst**
  - [x] 0.1 Repository, Workspace, Werkzeuge
  - [x] 0.2 Tauri-Hülle mit leerem Popover und Browser-Vorschau
- **MVP 1: Erfassen, Überblick, kurze Wege**
  - [x] 1.1 Core: Modell, Invarianten, Übergänge, Undo
  - [x] 1.2 Core: Speichern und Laden
  - [x] 1.3 Core: Schnelleingabe-Parser, Smart-Labels, Rang der Arbeitsmittel
  - [ ] 1.4 Tray, Popover-Fenster, Einzelinstanz, `--toggle`
  - [x] 1.5 Frontend-Grundlage: Rollen-Theming, Bausteine, Rückmeldung
  - [x] 1.6 Schnelleingabe
  - [x] 1.7 Arbeitskarte
  - [x] 1.8 Queue und Tastatur
  - [x] 1.9 Kurze Wege: Öffnen, `…`-Menü, `Alt+n`, `Strg+V`, Tray-Menü
  - [x] 1.10 Suche
  - [ ] 1.11 Pakete, Autostart, Alltagstest
- **MVP 2: Drei Geschwindigkeiten, Delegation, Themen**
  - [x] 2.1 Zwischendurch-Leiste
  - [x] 2.2 Ideen (vorgezogen vor 1.11, damit der Alltagstest sie hat)
  - [x] 2.3 Delegiert (vorgezogen vor 1.11)
  - [x] 2.4 Ausarbeitung (Split-View)
  - [ ] 2.5 Ablegen ins Popover, CLI `add`, `open`, `current`, `list`
  - [ ] 2.6 Themen: über das be spex kit (D38); Auswahl gebaut, eigene Theme-Dateien offen
  - [ ] 2.7 MCP-Server für KI-Assistenten
- **MVP 3: Linux-Feinschliff**
  - [ ] 3.1 `tasqlet capture` (Markierung übernehmen)
  - [ ] 3.2 Globale Tastenkürzel (X11-Plugin, Portal *GlobalShortcuts*)
  - [ ] 3.3 Export, AUR, Flatpak prüfen
  - [ ] 3.4 Übergabe aus Maillet

---

## Phase 0: Gerüst

### 0.1 Repository, Workspace, Werkzeuge

- Cargo-Workspace mit `crates/tasqlet-core` und `src-tauri`, Vite-Projekt in `src/` (nach E1).
- `rustfmt`, `clippy`, `eslint`/`svelte-check`, `vitest`. Skripte: `npm run dev`, `check`, `test`.
- CI (GitHub Actions): Format, Lint, Tests für Core und Frontend.
- `README.md` (kurz), `LICENSE` (MIT wie die anderen). `CLAUDE.md` gibt es schon. Sobald Code da ist, um
  eine Karte der Dateien und die Build-Befehle ergänzen.

**Abnahme:** `cargo test` und `npm test` laufen grün in der CI, auch wenn es noch keine echten Tests gibt.

### 0.2 Tauri-Hülle mit leerem Popover und Browser-Vorschau

- Tauri 2 mit einem rahmenlosen Fenster (470 px), das beim Start versteckt vorbereitet wird.
- `src/backend.ts` mit Mock-Schalter. `npm run dev` zeigt das Popover im Browser mit Beispieldaten aus
  dem Dummy.

**Abnahme:** Die App startet, das Fenster lässt sich zeigen und verstecken. Die Vorschau läuft im Browser.

## MVP 1: Erfassen, Überblick, kurze Wege

### 1.1 Core: Modell, Invarianten, Übergänge, Undo

Konzept: 2.1, 2.3, 2.4.

- Typen `Item`, `State`, `Resource`, `ChecklistItem`, `Store` (mit `on_break`), IDs als ULID.
- Übergänge als Methoden: übernehmen, als Nächstes, verschieben, erledigt, delegieren, zurück in Fokus,
  befördern, Pause/Weiter, Titel aus der Notiz.
- `last_active` und „Bereit“ nach Erledigt (kein Auto-Start).
- Undo-Verlauf (~50 Schritte) als Schnappschüsse oder inverse Operationen.
- Tests: jede Zeile der Tabelle 2.4, alle Invarianten 2.3 (eigenschaftsbasiert mit `proptest`: beliebige
  Aktionsfolgen verletzen keine Invariante).

**Abnahme:** Testabdeckung der Übergänge vollständig, `proptest` grün.

### 1.2 Core: Speichern und Laden

Konzept: 2.2, 2.5.

- `~/.local/share/tasqlet/items.json` (XDG über `dirs`), Archiv `archive/<Jahr>.json` für `done`.
- Atomar schreiben, 300 ms bündeln, `.bak`. Feld `schema` und eine Migrations-Schnittstelle.
- Änderungen von außen erkennen (`notify`) und neu laden, ohne eigene Schreibvorgänge als fremd zu werten.
- Tests: Absturz während des Schreibens (temporäre Datei bleibt liegen, Original heil), Round-Trip,
  kaputte Datei → `.bak` wird angeboten.

**Abnahme:** Daten überleben Neustart und `kill -9` mitten im Schreiben.

### 1.3 Core: Schnelleingabe-Parser, Smart-Labels, Rang der Arbeitsmittel

Konzept: 3.1, 5.

- Parser für `@Name`, `!heute|morgen|übermorgen|mo…so|TT.MM.[JJJJ]` und URLs/Pfade im Text. Ergebnis:
  Titel, Datum, Empfänger, Arbeitsmittel.
- Smart-Labels: Jira, GitHub-PR, GitHub-Issue, Datei, `.eml`/`mailto:`/`mid:`, Host.
- Rang: `used_at` absteigend, sonst Reihenfolge des Hinzufügens. Die Ziffer für `Alt+n` ist der feste
  Index.
- Tests mit Tabellen aus Eingabe → erwartetem Ergebnis (Beispiele aus dem Dummy übernehmen).

**Abnahme:** Alle Beispiele aus `KONZEPT.md` 3.1 und 5 als Tests grün.

### 1.4 Tray, Popover-Fenster, Einzelinstanz, `--toggle`

Konzept: 4, 8. Vorlagen: `../Maillet/src-tauri/src/app/tray.rs`, `windows.rs`, `shortcut.rs`;
`../CodeDeck/src-tauri/src/hub.rs`.

- Tray mit `ksni`: Symbol mit Zahl, Punkt (in Arbeit), Tasse (Pause), selbst gezeichnet. Tooltip.
  Menü: Öffnen, Schnell erfassen, Pause, Aktuelle erledigt, Beenden. Erste Menüzeile zeigt den Stand.
- Linksklick öffnet oder schließt das Popover. X11 neben dem Symbol, Wayland in Ecke oder Mitte
  (einstellbar). Fokusverlust schließt das Popover.
- Einzelinstanz über einen Unix-Socket. `tasqlet --toggle` holt die laufende Instanz.
- Hinweis, wenn kein Tray gefunden wird (GNOME ohne Erweiterung).

**Abnahme:** Auf Cinnamon (X11) und auf einem Wayland-Desktop öffnet ein Klick aufs Symbol oder eine auf
`tasqlet --toggle` gelegte Taste das Popover ohne spürbare Verzögerung.

**Stand 2026-11-25:** umgesetzt (D24), Tests grün, Release-Binary baut. Offen: die Abnahme am Desktop
(Cinnamon und Wayland) durch den Nutzer. Den sichtbaren Hinweis „kein Tray“ und die Anzeige eines
Ladefehlers rendert das Popover erst mit 1.5 (`tray_missing`, `load_error` im Snapshot).

### 1.5 Frontend-Grundlage: Rollen-Theming, Bausteine, Rückmeldung

Konzept: 6, 7.1. Vorlage: CSS des Dummys, `../CodeDeck/src/palette.ts`.

- CSS-Variablen `--tq-*` für alle Rollen inklusive `focus-*` und `chrome-*` (zunächst nur Stufe
  „ruhig“ / „eins mit Inhalt“). Paletten Papier und Graphit, „Automatisch“ folgt dem System.
- Zusätzlich Fjord mit den Stufen „betont“ und „abgesetzt“: das ist der Vorschlag **Leitstand**, der
  bevorzugte Stil (D19). Die Bausteinseite zeigt ihn mit.
- Bausteine: Reiter, Chip, Zeile mit Werkzeugen beim Überfahren, Abschnitt mit Label und `+`, Toast mit
  „Rückgängig“, Symbole (Lucide).
- i18n-Grundgerüst (de, en).
- Hinweise aus dem Snapshot anzeigen: `tray_missing` (GNOME-Erweiterung nennen) und `load_error`
  (Sicherung anbieten, `Storage::restore_backup`).

**Abnahme:** Eine Bausteinseite in der Vorschau, hell und dunkel, entspricht dem Dummy.

**Stand 2026-11-25:** Bausteinseite unter `http://localhost:1430/#bausteine` (Leitstand, Automatisch hell
und dunkel, Papier, Graphit), Stile aus dem CSS des Dummys (D25).

### 1.6 Schnelleingabe

Konzept: 3.1, 3.2 (`Strg+V` ohne Ziel).

- Feld beim Öffnen fokussiert. Legende nach Dringlichkeit: Jetzt › Fokus › Zwischendurch › Idee.
- Live-Anzeige erkannter Kürzel über den Parser aus 1.3.
- Die vier Tasten, die Rückmeldung mit Undo, `↓` in die Queue.
- Solange MVP 2 fehlt, landen Ideen und Zwischendurch nur in den Daten (Zähler am Reiter sichtbar).
- Jira-Keys lernen (`KONZEPT.md` 5, D26): Core lernt aus `…/browse/KEY-123` das Projekt und seine
  Basis-URL (`~/.config/tasqlet/jira.json`, atomar geschrieben), der Parser macht bekannte nackte Keys
  zu Arbeitsmitteln (Key bleibt im Titel). Tests: lernen, erkennen, unbekanntes Projekt bleibt Text,
  zwei Jira-Instanzen mit verschiedenen Projekten.

**Abnahme:** Alle vier Ziele, `@Name`, `!fr` und ein reiner Link funktionieren mit Tastatur allein. Nach
einem Jira-Link wird ein nackter Key desselben Projekts zum Chip.

**Stand 2026-11-25:** umgesetzt (D27). In der Browser-Vorschau geprüft (Erkennung, Legende, alle vier
Tasten, Rückgängig); am Desktop noch nicht ausprobiert.

### 1.7 Arbeitskarte

Konzept: 6.1, 2.1 (Bereit, Pause).

- Aufbau nach D20 (Entwurf K2, Vorlage `docs/entwuerfe/arbeitskarte-kopf.html`, weicht hier bewusst vom
  Dummy ab): keine Kopfzeile, der Titel steht oben. Rechts oben ein runder Knopf, `✓` Erledigt, bei
  „Bereit“ ausgefülltes `▶`. Beim Überfahren der Karte oder bei Tastaturfokus darin erscheinen
  darunter kleine runde Knöpfe: Tasse (Pause; bei „Bereit“ stattdessen `✓`), `➔` (einfaches
  Delegieren per Eingabe), `…` (zurück in die Queue, zu den Ideen).
- Klick auf den Titel öffnet die Ausarbeitung (zunächst ohne Funktion bis 2.4); Umbenennen per
  Doppelklick. Fußleiste: Status („In Arbeit“/„Bereit“), notiertes Datum, „Ausarbeiten“. Die Leiste
  nimmt später weitere Funktionen auf.
- Arbeitsmittel (drei wichtigste und `…`, siehe 1.9), Checkliste mit `+` unter dem Label, Notiz mit
  wachsender Höhe und Autosave.
- Pausenansicht mit „Weiter“.

**Abnahme:** Ein kompletter Durchlauf (übernehmen, Schritte abhaken, Notiz schreiben, Pause, weiter,
erledigt, #2 steht auf „Bereit“) sieht aus und verhält sich wie im Dummy, die Karte selbst wie
in D20.

### 1.8 Queue und Tastatur

Konzept: 6.1, 6.6, 2.4.

- Einzeilige Zeilen mit Indikatoren und Werkzeugen beim Überfahren.
- Sortieren per Pointer am Griff (nicht HTML5-DnD), `Alt+↑/↓`, `Alt+Pos1`, `⏎`, `↑/↓`.
- `Strg+F`, `Strg+Z`, `Esc` stufenweise. (`Strg+Z` und `Esc` gebaut; `Strg+F` kommt mit 1.10, `Strg+E`
  mit 2.4, ebenso das `⤢` der Zeile.)
- Hinweis ab 8 Einträgen.

**Abnahme:** Die ganze Tastatur-Tabelle 6.6 (ohne Ausarbeitung) funktioniert ohne Maus.

### 1.9 Kurze Wege

Konzept: 5.

- Öffnen über `xdg-open` (ohne Plugin, D30), `used_at` setzen.
- Drei Chips, `…`-Menü mit allen, „Alle öffnen“, „Link oder Datei hinzufügen“. Ziffern am Chip,
  `Alt+1…9`.
- `Strg+V` über einer Zeile oder der Karte hängt an.
- `🔗` an Zeilen klappt deren Arbeitsmittel aus.
- Tray-Menü: Arbeitsmittel der aktuellen Aufgabe und „Alle öffnen“.

**Abnahme:** Jeder Weg aus der Tabelle in `KONZEPT.md` 5 öffnet das richtige Ziel.

### 1.10 Suche

Konzept: 6.4.

- Volltext im Speicher über Offenes und das Archiv (Archiv bei Bedarf laden). Markierte Treffer,
  „Zuletzt erledigt“ ohne Suchbegriff.
- Ein Klick zeigt den Eintrag (bis 2.4 einfache Detailansicht oder Übernehmen).

**Abnahme:** Eine Suche über 5 000 Einträge antwortet ohne spürbare Verzögerung (unter 50 ms).

### 1.11 Pakete, Autostart, Alltagstest

Vorlage: `../Maillet` (Packaging), `../aur`.

- `.deb` und AppImage, `.desktop`-Datei, Symbol, XDG-Autostart anbieten. (Release-Workflow, Paket
  und Eintrag im APT-Repository stehen, D35; Autostart in den Einstellungen, D33.)
- Eine Woche im Alltag nutzen, Befunde in `docs/ENTSCHEIDUNGEN.md` und als Schritte hier.
  Erster Befund (D36): mehr Tasten (Reiter, Pause, Alle öffnen, Schnelleingabe), globale Taste in den
  Einstellungen (X11), leere Abschnitte der Karte einzeilig.

**Abnahme:** Installiert auf einem frischen System, Autostart klappt, eine Woche ohne Datenverlust.

## MVP 2: Drei Geschwindigkeiten, Delegation, Themen

### 2.1 Zwischendurch-Leiste

Konzept: 6.1 (Zwischendurch). ⚡ als Symbol mit Tooltip, Pillen (drei und `+n`), Klick übernimmt,
Mittelklick und `✓` haken ab.
**Abnahme:** wie im Dummy, inklusive Undo.

### 2.2 Ideen

Konzept: 6.2. Karten, neueste oben, drei Stichpunkte und „+n weitere“, Brainstorm-Zeile, `→ Fokus`,
`⚡ Zw.`.
**Abnahme:** Aus einem Stichwort wird über mehrere Brainstorm-Zeilen eine Aufgabe im Fokus, nichts geht
verloren.

### 2.3 Delegiert

Konzept: 6.3. Gruppierung nach `@Name`, Übergabe- und notiertes Datum, Notiz, Checkliste, „Zurück in
Fokus“, „Erledigt“. Namensvorschläge aus bisherigen Empfängern.
**Abnahme:** Delegieren aus Schnelleingabe, Arbeitskarte und Ausarbeitung landet hier.

### 2.4 Ausarbeitung (Split-View)

Konzept: 6.5. Fenster wird doppelt so breit, links die Listen, rechts der Editor mit Status, Datum,
Delegieren, Abschnitten. `Strg+E`, `⤢`, `Esc`.
**Abnahme:** Jeder Eintrag aus jeder Liste lässt sich hier vollständig bearbeiten.

### 2.5 Ablegen ins Popover, CLI

Konzept: 3.2, 9.

- Drop von Dateien und Links auf Zeile, Karte oder Idee (Drop-Ereignis von Tauri, echte Pfade).
- CLI über den Socket: `tasqlet add "…" [--focus|--now|--inter] [--link URL]`, `tasqlet open [n]`,
  `tasqlet current [--json]`, `tasqlet list [--state …] [--json]`. Ohne laufende App direkt auf die
  Datei mit `flock`.

**Abnahme:** Ein Skript legt eine Aufgabe mit Link an, sie erscheint sofort im Popover.
Stand: gebaut (D37), dazu der Knopf „Anheften“; ohne laufende App hält die CLI die Einzelinstanz-Sperre
statt eines eigenen `flock`. Abnahme am Desktop steht aus.

### 2.6 Themen

Konzept: 7. Grundlage seit D38: `@bespex/themes` aus `../kit` (dort `PLAN.md` K1–K4).

- Farbthema und Stil kommen aus dem Kit: 12 Themen plus „Automatisch“, vier Stile, sieben Achsen.
  Tasqlets Achsen „aktuelle Aufgabe“ und „Bedienelemente“ sind dort `emphasis` und `chrome`, die
  Vorschläge sind Paare aus Thema und Stil. Tasqlet bietet die Achsen `shape`, `structure`, `chrome`
  und `emphasis` an.
- Einstellungsdialog: Abschnitt „Erscheinungsbild“ mit der Auswahl des Kits (`<sx-appearance>`),
  wirkt sofort, gespeichert als `appearance` in `settings.json`. Vorgabe wie bisher Leitstand
  (Fjord, Leisten abgesetzt, Aufgabe betont).
- Kontrastprüfung: im Kit über alle Themen und alle farbwirksamen Stilstufen (`npm run check` dort).
- Offen: eigene Theme-Dateien der Nutzer in `~/.config/tasqlet/themes/`, geprüft mit `check` des
  Kits und mit Hinweis abgelehnt, wenn sie die Prüfung verfehlen (Kit-Brainstorm „eigene Themen“).

**Abnahme:** Die Auswahl in den Einstellungen wechselt Thema und Stil sofort und behält sie nach
einem Neustart; die Bausteinseite (`#bausteine`) ist in jeder gezeigten Kombination lesbar. Gebaut
2026-11-25, Abnahme am Desktop steht aus. Ein absichtlich schlechtes eigenes Thema wird abgelehnt
(offen, mit den Theme-Dateien).

### 2.7 MCP-Server für KI-Assistenten

Konzept: 3.3, 9.

- Unterbefehl `tasqlet mcp` (stdio, `rmcp`). Spricht über den Socket aus 2.5 mit der laufenden App,
  sonst direkt mit der Datei (`flock`).
- Werkzeuge und Prompt wie in `KONZEPT.md` 3.3. Lesen und Ergänzen, nicht übernehmen, erledigen,
  umsortieren oder löschen.
- Änderungen der KI laufen durch dieselbe `Session` (Undo) und erscheinen im Popover als Rückmeldung
  mit Kennzeichen „KI“ und „Rückgängig“.
- Kurze Anleitung im README: Eintrag für Claude Code (`claude mcp add tasqlet -- tasqlet mcp`) und
  Claude Desktop.

**Abnahme:** Ein KI-Assistent liest die aktuelle Aufgabe, zerlegt sie in Schritte und schreibt einen
Befund in die Notiz; alles erscheint sofort im Popover und lässt sich rückgängig machen.

## MVP 3: Linux-Feinschliff

### 3.1 `tasqlet capture`

Konzept: 3.2, 8. Popover öffnen, dann Primary Selection lesen. Text in die Schnelleingabe, URL oder Pfad
an die aktuelle Aufgabe.
**Abnahme:** funktioniert unter X11 und auf KDE und GNOME unter Wayland.

### 3.2 Globale Tastenkürzel

X11 über `tauri-plugin-global-shortcut`, Wayland über das Portal *GlobalShortcuts* (`ashpd`). Fällt
beides aus, bleibt `--toggle`. (Der X11-Teil mit Aufnahme in den Einstellungen steht, D36.)
**Abnahme:** In den Einstellungen festgelegte Taste wirkt auf allen drei Testdesktops oder zeigt, warum
nicht.

### 3.3 Export, AUR, Flatpak

`tasqlet export --markdown`, AUR-Paket (`../aur`), Flatpak-Machbarkeit (Tray, `xdg-open`) prüfen und
entscheiden.

### 3.4 Übergabe aus Maillet

Konzept: 10.

- Maillet: „An Tasqlet“ ruft `tasqlet add "<Betreff>" --link <eml>` auf (Änderung in `../Maillet`).
- HatRack spielt vorerst keine Rolle.

**Abnahme:** Eine Mail aus Maillet erscheint als Aufgabe mit Betreff als Titel und der Mail als Chip.

## Testdesktops

Cinnamon (X11, Linux Mint) als Hauptsystem, dazu KDE Plasma (Wayland) und GNOME (Wayland, mit und ohne
AppIndicator-Erweiterung). Jeder Schritt mit Desktop-Bezug (1.4, 1.9, 1.11, 3.x) wird auf allen dreien
abgenommen.
