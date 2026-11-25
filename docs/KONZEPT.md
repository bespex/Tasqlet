# Tasqlet: Konzept

Verbindliche Vorgabe für die Umsetzung, Stand 2026-11-24. Das Zielbild zeigt der Click-Dummy in
`docs/click-dummy/` (Abschnitt 11). Wie es umgesetzt wird, steht in `docs/PLAN.md`. Entwurf und Review,
aus denen dieses Konzept entstanden ist, liegen in `docs/archiv/` (nur zur Nachvollziehbarkeit).

## 1. Was Tasqlet ist

Tasqlet ist ein persönliches Fokus- und Notiz-Werkzeug im System-Tray des Linux-Desktops, Teil der
`bespex`-Familie: klein, sofort da, lokal, löst genau ein Problem.

**Das Problem:** Im Arbeitsalltag laufen drei Geschwindigkeiten nebeneinander. Es gibt Dringendes in
wechselnder Reihenfolge, kleine Dinge für zwischendurch und rohe Ideen, die über Tage wachsen. Zur
aktuellen Aufgabe gehört außerdem ihr Arbeitskontext (Ticket, Mail, Dateien, Links), dazu ein Ort zum
Strukturieren (Checkliste) und zum Merken (Notiz).

**Schwerpunkt** sind drei Dinge, alles andere ordnet sich ihnen unter:

1. **Einfach erfassen**: ein Gedanke, ein Link, eine Aufgabe in Sekunden, ohne Maus.
2. **Schnell überblicken**: Was ist dran, was kommt danach? Möglichst schon ohne das Popover zu öffnen.
3. **Kurze Wege zu den Arbeitsmitteln**: ein Klick oder eine Taste bis zum Ticket, zur Mail, zur Datei.

### 1.1 Leitlinien

- **Ein Objekt statt künstlicher Trennung.** Notiz und Aufgabe sind dasselbe `Item` in unterschiedlichen
  Reifegraden. Beim Befördern geht nichts verloren.
- **Genau ein Fokus.** Höchstens eine Aufgabe ist „in Arbeit“. Nur sie ist aufgeklappt, alles andere
  bleibt ruhig.
- **Organisieren, nicht überwachen.** Tasqlet misst keine Zeit und überwacht keine Termine. „In Arbeit“
  heißt nur: Das ist gerade mein Fokus. Zwischendurch etwas anderes zu tun, weil man warten muss oder
  eine andere Aufgabe den nächsten Schritt braucht, ist normaler Alltag. Das Tool zeigt das ruhig an,
  ohne es als Unterbrechung zu werten.
- **Instant und leicht.** Das Popover öffnet in unter 100 ms. Der Prozess läuft im Tray, das Fenster ist
  vorbereitet.
- **Lokal und transparent.** Die Daten liegen in einer lesbaren JSON-Datei. Es gibt keine Cloud und
  keine Datenbank.
- **Tastatur zuerst, Maus gleichwertig.** Jede häufige Aktion hat eine Taste.

### 1.2 Nicht-Ziele

- Keine Zeiterfassung, keine Laufzeit-Anzeige, keine Auswertung.
- Keine Fristen: kein Einfärben fälliger Termine, keine Erinnerungen, keine Sortierung nach Datum.
- Kein Kalender, keine Synchronisation (CalDAV, ICS).
- Kein Team-Werkzeug: keine Sprints, Boards, Story Points, Mehrbenutzer.
- Kein Wiki: keine Ordnerbäume, kein Rich-Text, keine Backlinks.

## 2. Datenmodell

### 2.1 Zustände

| `state` | Bedeutung | Ort in der Oberfläche |
|---|---|---|
| `idea` | roher Gedanke, wachsende Notiz | Reiter **Ideen** |
| `focus` | dringend, in fester Reihenfolge (#1, #2, …) | Reiter **Fokus**, Hauptliste |
| `intermezzo` | klein, ohne Eile | Reiter **Fokus**, Leiste unten (⚡) |
| `delegated` | abgegeben an `@Name`, im Blick behalten | Reiter **Delegiert** |
| `done` | erledigt, bleibt durchsuchbar | nur in der Suche |

Innerhalb von `focus`:

- **`active`**: höchstens ein Eintrag, steht auf #1 und ist als **Arbeitskarte** aufgeklappt.
- **„Bereit“**: #1 ist nicht `active`, etwa nach „Erledigt“. Die Karte ist aufgeklappt und zeigt `▶`.
  Was als Nächstes dran ist, entscheidet man selbst. Es gibt keinen Auto-Start.
- **`last_active`**: Wer eine andere Aufgabe übernimmt, findet die verlassene auf #2 mit
  **„↩ zuletzt dran“**. Es gibt höchstens einen solchen Eintrag. Die Markierung verschwindet, sobald
  man die Aufgabe wieder aufnimmt oder eine andere verlässt.
- **Pause (Kaffeetasse)**: ein globaler Zustand `on_break`. Die Arbeitskarte tritt zurück, das Popover
  zeigt „Pause. Danach weiter mit …“, das Tray-Symbol eine Tasse. „Weiter“ nimmt dieselbe Aufgabe
  wieder auf. Wer eine andere Aufgabe startet, beendet die Pause.

### 2.2 Datei

```jsonc
// ~/.local/share/tasqlet/items.json      (Erledigtes in archive/<Jahr>.json)
{
  "schema": 1,
  "on_break": false,
  "items": [{
    "id": "01JD…",                   // ULID
    "title": "CORE-418: Webhook-Authentifizierung umstellen",
    "state": "focus",                // idea | focus | intermezzo | delegated | done
    "active": true,                  // nur bei focus, höchstens einer
    "last_active": false,            // nur bei focus, höchstens einer
    "order": 0,                      // je Liste lückenlos: Fokus, Zwischendurch (Ideen: neueste oben)
    "note": "Staging nutzt noch X-Legacy …",
    "checklist": [{ "id": "…", "text": "Alten Token-Flow prüfen", "done": true }],
    "resources": [{ "id": "…", "type": "url", "target": "https://jira…/browse/CORE-418",
                    "label": null,   // null = Smart-Label
                    "used_at": null }],
    "date": "2026-11-27",            // nur notiert, keine Auswertung
    "delegated_to": null, "delegated_at": null,
    "tags": [],                      // später: Kontext für HatRack
    "created_at": "…", "updated_at": "…", "done_at": null
  }]
}
```

- `resources[].type`: `url`, `file` (Datei oder Ordner), `mail` (`.eml`, `mailto:`, `mid:`).
- Die Reihenfolge von `checklist` und `resources` ist die Reihenfolge im Array, also die des
  Hinzufügens.
- Ein Titel ist nicht Pflicht. Wird ein Eintrag ohne Titel befördert, wird die erste Notizzeile zum
  Titel.

### 2.3 Invarianten (im Core, mit Tests)

- Höchstens ein `active` und höchstens ein `last_active`. Beide gibt es nur bei `state = focus`.
- `order` ist je Liste lückenlos ab 0. Ein `active` Eintrag hat `order = 0`.
- Ein Wechsel aus `focus` heraus setzt `active` und `last_active` zurück.
- `on_break = true` nur, wenn es ein `active` gibt.

### 2.4 Übergänge

| Aktion | Wirkung |
|---|---|
| **Übernehmen** (`▶`, Enter auf Zeile, Klick auf Pille) | Eintrag wird `active` auf #1. Die bisherige #1 rückt auf #2, war sie `active`, bekommt sie `last_active`. Aus `intermezzo` oder `idea` wird `focus`. |
| **Als Nächstes** (`⤒`, `Alt+Pos1`) | Eintrag auf #2, die aktuelle Arbeit bleibt. |
| **Verschieben** (Griff, `Alt+↑/↓`) | nur innerhalb von #2 … n. |
| **Erledigt** (`✓`, Mittelklick auf Pille) | `done`, `done_at`. Steht er auf #1, rückt #2 als „Bereit“ nach. |
| **Delegieren** (`➔`, `@Name`) | `delegated`, `delegated_to`, `delegated_at`. |
| **Zurück in Fokus** | ans Ende der Fokus-Liste. |
| **→ Fokus / ⚡ Zw.** (Ideen) | ans Ende der jeweiligen Liste. |
| **Pause / Weiter** | `on_break` an oder aus. |

**Jeder Zustandswechsel ist rückgängig zu machen**: Die Rückmeldung (Toast) bietet „Rückgängig“, dazu
gibt es `Strg+Z`. Der Verlauf liegt im Speicher und umfasst etwa 50 Schritte.

### 2.5 Speichern

Atomar schreiben (temporäre Datei, `fsync`, `rename`), 300 ms gebündelt, die vorige Fassung bleibt als
`items.json.bak`. Wird die Datei von außen geändert (CLI, Editor), lädt Tasqlet sie neu. `schema`
erlaubt spätere Migrationen. Für Menschen gibt es `tasqlet export --markdown`.

## 3. Erfassen

### 3.1 Schnelleingabe

Das Eingabefeld oben ist beim Öffnen fokussiert. Unter dem Feld stehen die Ziele, **sortiert nach
Dringlichkeit**:

| Taste | Ziel | Wirkung |
|---|---|---|
| `Strg ⇧ ⏎` | **Jetzt** | sofort `active` auf #1, die bisherige rückt auf #2 |
| `Strg ⏎` | **Fokus** | ans Ende der Fokus-Liste, die aktuelle Arbeit bleibt |
| `Alt ⏎` | **Zwischendurch** | in die Leiste unten |
| `⏎` | **Idee** | in die Ideen, ohne Dringlichkeit |

Jedes Speichern meldet sich kurz zurück, mit „Rückgängig“ (z. B. „Als Idee gespeichert (Strg ⏎ = Fokus)“).

**Kürzel beim Tippen** werden live erkannt und unter dem Feld angezeigt:

- `@Name`: delegiert direkt, Enter legt den Eintrag unter **Delegiert** ab.
- `!heute`, `!morgen`, `!fr`, `!28.11.`: notiert ein Datum.
- Eine URL oder ein Pfad wird Arbeitsmittel und verschwindet aus dem Titel.
- Ein Link allein ergibt einen Eintrag mit Smart-Label als Titel (`PR #42`).

### 3.2 Weitere Wege

| Weg | Wie | Stufe |
|---|---|---|
| `Strg+V` ins offene Popover | Über einer Aufgabe (überfahren oder ausgewählt): Link oder Pfad wird angehängt. Ohne Ziel kommt der Inhalt in die Schnelleingabe. Hat ein Textfeld den Fokus, wird normal eingefügt. | MVP 1 |
| Globaler Hotkey | Öffnet das Popover mit fokussierter Eingabe (`tasqlet --toggle`) | MVP 1 |
| Ablegen | Dateien und Links aus Dateimanager oder Browser auf eine Aufgabe im Popover ziehen; das 📌 in der Kopfzeile hält das Popover dafür offen. Auf eine freie Stelle: in die Schnelleingabe (D37) | MVP 2 |
| CLI | `tasqlet add "…" [--now|--focus|--inter|--idea] [--link URL]` (ohne Option ans Ende des Fokus), `open [n]`, `current`, `list` (D37) | MVP 2 |
| Markierung übernehmen | `tasqlet capture`: markierter Text kommt in die Schnelleingabe (Enter bestätigt), eine markierte URL oder ein Pfad an die aktuelle Aufgabe | MVP 3 |

### 3.3 KI-Assistenten über MCP

Aufgaben werden zunehmend mit KI erledigt. Ein MCP-Server (`tasqlet mcp`, Transport stdio) gibt einem
KI-Assistenten (Claude Code, Claude Desktop, …) Zugriff auf Tasqlet: Er kann eine Aufgabe **prüfen**,
den Menschen dabei **unterstützen** und Ergebnisse zurückschreiben. Der Server ist ein Unterbefehl der
App, spricht über den Socket mit der laufenden App (ohne App direkt mit der Datei, wie die CLI) und
bleibt rein lokal.

| Art | Werkzeuge | Wirkung |
|---|---|---|
| Lesen | `current_task`, `get_task`, `list_tasks`, `search` | aktuelle Aufgabe mit Checkliste, Notiz, Arbeitsmitteln; Listen nach Zustand; Suche inklusive Erledigtem |
| Ergänzen | `add_task`, `add_checklist_items`, `append_note`, `add_resource`, `check_item` | neue Aufgabe (wie die Schnelleingabe), Aufgabe in Schritte zerlegen, Befund in die Notiz, Link oder Datei anhängen, Schritt abhaken |
| Vorlage | Prompt `review_task` | „Prüfe diese Aufgabe: Ist sie klar, fehlen Schritte oder Arbeitsmittel?“ |

- **Was die KI nicht darf:** Aufgaben übernehmen, erledigen, umsortieren, delegieren oder löschen. Was
  als Nächstes dran ist und was fertig ist, entscheidet der Mensch (D4). Neue Aufgaben landen höchstens
  am Ende der Fokus-Liste, nie auf #1.
- **Sichtbar und umkehrbar:** Jede Änderung der KI erscheint im Popover als Rückmeldung mit Kennzeichen
  „KI“ und „Rückgängig“ und liegt im normalen Undo-Verlauf.
- Ohne Zeitbezug wie überall (D1, D2): Die KI bekommt keine Laufzeiten, Fristen werden nicht ausgewertet.

## 4. Überblick

- **Ohne Öffnen.** Das Tray-Symbol zeigt die Zahl der Einträge im Fokus. Ein Punkt bedeutet „in
  Arbeit“, eine Tasse „Pause“. Der Tooltip zeigt: `▶ CORE-418: Webhook-Auth (1/3) · Danach:
  Release-Notes`. Wo der Desktop keine Tooltips zeigt (GNOME), steht dasselbe als erste Zeile im
  Tray-Menü.
- **Im Popover.** Oben die eine Arbeitskarte, darunter einzeilige Einträge, unten die Pillen von
  Zwischendurch. Die Zahlen stehen an den Reitern.
- **Leise Hinweise statt Druck.** Ab 8 Einträgen im Fokus erscheint der Hinweis „Ist das alles
  dringend?“, es gibt keine Sperre.

## 5. Kurze Wege zu den Arbeitsmitteln

- An der Arbeitskarte stehen **die drei wichtigsten** Arbeitsmittel als Chips. Wichtig heißt: zuletzt
  aufgerufen (`used_at`), sonst in der Reihenfolge des Hinzufügens.
- Der Chip **`…`** öffnet ein Menü mit allen Arbeitsmitteln (zuletzt benutzte oben), dazu „Alle öffnen“
  und „Link oder Datei hinzufügen“.
- `Alt+1` … `Alt+9` öffnet das n-te Arbeitsmittel der aktuellen Aufgabe. **Die Ziffer gehört fest zum
  Arbeitsmittel** (Reihenfolge des Hinzufügens) und steht klein am Chip. Sie wandert nicht mit, wenn
  sich die Anzeige umsortiert, `Alt+1` bleibt also meist das Ticket.
- Ein Rechtsklick aufs Tray-Symbol zeigt die Arbeitsmittel der aktuellen Aufgabe direkt im Menü, ohne
  das Popover.
- Das `🔗` an einer Zeile der Queue klappt deren Arbeitsmittel aus.
- Geöffnet wird über `xdg-open`. Jedes Öffnen setzt `used_at`.
- **Smart-Labels**: Jira `…/browse/CORE-418` → `CORE-418`, GitHub-PR → `PR #42`, GitHub-Issue →
  `repo#12`, Datei → Dateiname, `.eml` → `Mail: …`, sonst der Host. Beim Überfahren erscheint das
  volle Ziel.
- **Jira-Keys lernen (D26)**: Taucht ein Link wie `https://jira.firma.de/browse/NMS-19465` auf
  (Schnelleingabe, `Strg+V`, Ablegen), merkt sich Tasqlet das **Projekt**: `NMS` gehört zu
  `https://jira.firma.de`. Danach wird ein nackter Key `NMS-19470` in der Schnelleingabe zum
  Arbeitsmittel und bleibt im Titel stehen. In Titeln und Notizen, die als Text angezeigt werden
  (Queue, Ideen, Delegiert, Suche), sind bekannte Keys klickbar. Unbekannte Projekte bleiben Text, so
  werden `UTF-8` oder `SHA-256` nie zu Links. Die gelernten Projekte liegen in
  `~/.config/tasqlet/jira.json`; der Einstellungsdialog zeigt sie und kann jedes wieder vergessen.

## 6. Oberfläche

Das Popover ist rahmenlos, 700 px breit wie Maillets Fenster (D39), und in der Ausarbeitung breiter. Es gibt drei
Reiter (Fokus, Ideen, Delegiert) und die Suche (`🔍`, `Strg+F`).

### 6.1 Fokus

**Arbeitskarte (#1)**

- Keine Kopfzeile (D20). Oben steht der **Titel**. Ein Klick darauf öffnet die Ausarbeitung,
  ein Doppelklick benennt um.
- Rechts oben ein **runder Knopf**: `✓` Erledigt, bei „Bereit“ ein ausgefülltes `▶`. Beim Überfahren
  der Karte oder bei Tastaturfokus darin erscheinen darunter kleine runde Knöpfe: Tasse (Pause; bei
  „Bereit“ stattdessen `✓`), `➔` Delegieren, `…` (zurück in die Queue, zu den Ideen).
- Drei Abschnitte, getrennt durch Abstand und eine feine Linie. Das Label steht links:
  - **Arbeitsmittel**: drei Chips und `…` (Abschnitt 5).
  - **Checkliste**: Schritte zum Abhaken.
  - **Notiz**: wächst mit ihrem Inhalt, von zwei Zeilen bis etwa zwölf, danach scrollt sie. Sie
    speichert automatisch.
- **Ergänzen über ein `+` unter dem Label** von Arbeitsmittel und Checkliste. Ein Klick öffnet die
  Eingabe, `⏎` speichert und lässt sie für den nächsten Eintrag offen, `Esc` schließt sie. Es gibt keine
  dauerhaften „+ hinzufügen“-Zeilen.
- **Fußleiste** der Karte: Status („In Arbeit“ oder „Bereit“), notiertes Datum, „Ausarbeiten“. Sie
  nimmt später weitere Funktionen auf.

**Queue (#2 …)**

- Einzeilig: Nummer, Titel, Indikatoren (`↩ zuletzt dran`, Checklisten-Fortschritt `2/4`, `🔗` mit
  Anzahl, Datum).
- Die Werkzeuge erscheinen erst beim Überfahren oder bei Tastaturfokus: Griff zum Sortieren, `⤢`, `⤒`,
  `▶`.

**Zwischendurch (Fußleiste)**

- Nur ein ⚡-Symbol als Kennzeichen, Name und Anzahl stehen im Tooltip. Danach folgen Pillen, höchstens
  drei, dann `+n`.
- Klick auf eine Pille übernimmt sie, Mittelklick hakt sie ab. Beim Überfahren erscheint `✓` als
  sichtbarer Weg zum Abhaken.

### 6.2 Ideen

Karten, die neueste oben, mit dezentem Alter. Jede Karte zeigt höchstens 3 Stichpunkte der Notiz, dann
„+n weitere“, außerdem Arbeitsmittel und den Checklisten-Fortschritt. Eine Einzeile „+ Brainstorm-
Gedanke anfügen …“ hängt mit `⏎` einen Stichpunkt an (ist es ein Link, wird er Arbeitsmittel). Dazu
kommen `→ Fokus`, `⚡ Zw.` und `⤢ Ausbau`.

### 6.3 Delegiert

Gruppiert nach `@Name`. Jeder Eintrag zeigt das Übergabedatum, das notierte Datum (z. B. wann man
nachfragen will, ohne Erinnerung), Notiz, Checkliste und Arbeitsmittel. Aktionen: „Zurück in Fokus“ und
„Erledigt“.

### 6.4 Suche

Volltext über Titel, Notizen, Checklisten, Arbeitsmittel (Label und Ziel) und `@Name`, auch über
Erledigtes. Treffer werden markiert. Ohne Suchbegriff erscheint „Zuletzt erledigt“. Ein Klick öffnet die
Ausarbeitung.

### 6.5 Ausarbeitung (Split-View, `Strg+E`, `⤢`)

Links alle offenen Einträge nach Liste gruppiert, rechts der Editor: Titel, Status, Datum, Delegieren,
dann Arbeitsmittel, Checkliste und Notiz mit `+` an den Überschriften. `Esc` oder `Strg+E` schließt.

### 6.6 Tastatur

| Wo | Taste | Wirkung |
|---|---|---|
| überall | `Strg+F` / `Strg+E` / `Strg+Z` | Suche / Ausarbeitung / Rückgängig |
| überall | `Alt+1` … `Alt+9` | Arbeitsmittel der aktuellen Aufgabe öffnen |
| überall | `Strg+1` / `Strg+2` / `Strg+3`, `Strg+Tab` | Reiter Fokus / Ideen / Delegiert, nächster Reiter (`⇧` zurück) |
| überall | `Strg+P` / `Strg+O` / `Strg+L` | Pause und weiter / alle Arbeitsmittel öffnen / in die Schnelleingabe |
| global (X11) | in den Einstellungen festgelegt | Popover ein- und ausblenden (D36); sonst `tasqlet --toggle` |
| überall | `Esc` | schließt stufenweise: Menü oder Eingabe, Ausarbeitung, Suche, Popover |
| Schnelleingabe | `Strg ⇧ ⏎`, `Strg ⏎`, `Alt ⏎`, `⏎` | siehe 3.1 |
| Schnelleingabe (leer) | `↓` | in die Queue |
| Queue | `↑` / `↓` | Auswahl (`↑` auf der ersten Zeile zurück in die Eingabe) |
| Queue | `⏎` | übernehmen |
| Queue | `Alt+↑/↓`, `Alt+Pos1` | verschieben, als Nächstes |
| Queue | `Strg+V` | Link an die ausgewählte Zeile |

## 7. Farbthemen

> Stand 2026-11-25 (D38): Umgesetzt über das gemeinsame be spex kit (`../kit`, `PLAN.md` K1–K4).
> Paletten sind dort Farbthemen, die beiden Achsen sind die Stilachsen `emphasis` und `chrome`, die
> Rollen heißen `--sx-*`. Dieser Abschnitt bleibt als Herkunft der Achsen und der Kontrasttabelle.

Die Oberfläche kennt keine festen Farben, nur Rollen. Ein Thema besteht aus drei Teilen: einer
**Palette** und zwei **Achsen**.

### 7.1 Palette (Rollen)

| Rolle | Wofür | Mindestkontrast |
|---|---|---|
| `bg`, `card`, `field`, `hover` | Grund, Karten, Eingabefelder, Überfahren/Auswahl | – |
| `line` | feine Trennlinien | – |
| `line-strong` | Rahmen von Eingaben und Kontrollen | 3:1 zu `card` und `bg` |
| `ink` | Text | 7:1 |
| `ink-2` | Labels, Metadaten, Zahlen | 4,5:1 zu `bg`, `card`, `field` |
| `ink-3` | Platzhalter, Nebensächliches | 3:1 |
| `accent`, `on-accent`, `accent-soft` | „In Arbeit“, Tastaturfokus, „Jetzt“, `+` beim Überfahren | 4,5:1 |
| `accent-inv` | Akzent auf invertierten Flächen | 4,5:1 zu `ink` |
| `mark` | Treffer in der Suche | `ink` darauf 7:1 |

Der Akzent wird sparsam eingesetzt. Rückmeldungen (Toast) sind invertiert (`ink` als Fläche) und passen
zu jedem Thema.

### 7.2 Achsen

| Achse | Stufen | Wirkung |
|---|---|---|
| Aktuelle Aufgabe (`focus`) | ruhig · betont · Bühne | ruhig: feiner Akzentrand. betont: getönte Fläche, Akzentrand 2 px, Schatten, Queue-Text tritt zurück. Bühne: invertierte Karte |
| Bedienelemente (`chrome`) | eins mit Inhalt · abgesetzt · kontrastiert | Tabs, Schnelleingabe, Fußleiste, linke Spalte der Ausarbeitung: gleiche Fläche, leicht getönt mit kräftiger Trennlinie, oder invertiert |

Dafür gibt es zwei weitere Rollengruppen, `focus-*` und `chrome-*`, mit denselben Rollen wie die
Palette, dazu Randstärke, Schatten und Trennlinie. Ohne Angabe folgen sie der Palette. Die Stufen leiten
sie ab. Ein Thema darf jede Rolle direkt setzen. Auf invertierten Flächen erhalten native Kontrollen
(Checkbox, Datum) das umgekehrte Farbschema.

### 7.3 Lieferumfang und Prüfung

- Paletten: Automatisch (folgt hell/dunkel des Systems), Papier, Graphit, Fjord, Salbei, Kontrast. Die
  Werte stehen im Click-Dummy.
- Vorschläge: Bühne, Scheinwerfer, Werkbank, Leitstand, Atelier, Klar.
- Eigene Themen liegen als Datei in `~/.config/tasqlet/themes/`:

  ```jsonc
  { "name": "Bühne", "palette": "papier", "focus": "stage", "chrome": "flat",
    "override": { "focus-accent": "#ffb38a" } }
  ```

- Der Core prüft beim Laden alle Kontraste der Tabelle. Ein Thema, das sie verfehlt, wird mit Hinweis
  abgelehnt. Ein Testlauf prüft alle mitgelieferten Kombinationen aus Palette und beiden Achsen.

## 8. Linux-Integration

| Funktion | X11 (Cinnamon, KDE, XFCE) | KDE Wayland | GNOME Wayland | Umsetzung |
|---|---|---|---|---|
| Tray-Symbol | ✓ | ✓ | nur mit Erweiterung *AppIndicator* | `ksni`. Findet Tasqlet kein Tray, zeigt es beim ersten Start einen Hinweis (wie Maillet) |
| Popover am Tray-Symbol | ✓ (Koordinaten aus SNI `Activate`) | Position wird ignoriert | Position wird ignoriert | X11 am Symbol, Wayland Ecke oder Mitte (einstellbar), Logik aus Maillet `windows.rs` |
| Tooltip | ✓ | ✓ | ✗ | Stand zusätzlich als erste Menüzeile |
| Arbeitsmittel im Tray-Menü | ✓ | ✓ | ✓ | dynamisches SNI-Menü |
| Globaler Hotkey | `tauri-plugin-global-shortcut` | Portal *GlobalShortcuts* | Portal (neuere GNOME) | MVP 1: `tasqlet --toggle`, im Desktop auf eine Taste gelegt. Portal in MVP 3 |
| Markierung übernehmen | jederzeit | nur mit Fokus | nur mit Fokus | `tasqlet capture` öffnet erst das Popover und liest dann die Markierung |
| Drop aufs Tray-Symbol | ✗ | ✗ | ✗ | **gibt es nicht**, Drop nur ins Popover |
| Autostart | XDG-Autostart | ✓ | ✓ | Eintrag beim ersten Start anbieten |

## 9. Technik

| Schicht | Wahl |
|---|---|
| App-Hülle | Tauri 2 (Rust), wie Maillet und CodeDeck |
| Domäne | Crate `tasqlet-core` ohne Tauri: Modell, Übergänge, Undo-Verlauf, Store, Kürzel-Parser, Smart-Labels, Rang der Arbeitsmittel, Themen und Kontrastprüfung |
| Tray | `ksni` (StatusNotifierItem): Symbol mit Zahl, Punkt oder Tasse, Tooltip, dynamisches Menü |
| Frontend | Svelte 5 + TypeScript + Vite, Notiz als `<textarea>`, Sortieren mit eigener Pointer-Logik |
| Speicher | `serde_json`, `tempfile`, `notify` |
| Einzelinstanz & CLI | Unix-Socket in `$XDG_RUNTIME_DIR` (Muster CodeDeck-Hub). Läuft die App nicht, schreibt die CLI direkt in die Datei (`flock`) |
| MCP-Server | `tasqlet mcp` (stdio) mit `rmcp`, nutzt denselben Socket bzw. dieselbe Datei wie die CLI |
| Öffnen | `xdg-open` (`tauri-plugin-opener`) |
| i18n | Deutsch und Englisch |
| Pakete | `.deb`, AppImage, AUR. Flatpak später prüfen (Sandbox gegen `xdg-open` beliebiger Pfade) |

```
tasqlet/
├── Cargo.toml            # Workspace
├── crates/tasqlet-core/  # Domäne, ohne GUI, cargo test
├── src-tauri/            # Tray, Fenster, Socket, CLI-Subcommands, Commands → core
├── src/                  # Svelte: Ansichten, Bausteine, Themen
└── docs/                 # KONZEPT.md, PLAN.md, click-dummy/
```

Für ein Programm, das ständig im Tray liegt, wird das Fenster beim Schließen versteckt statt zerstört.
Der Speicherverbrauch wird früh gemessen.

## 10. Abgrenzung

- **Maillet** verwaltet eingehende Mails und hält die Inbox leer. Braucht eine Mail mehr, übergibt
  Maillet sie an Tasqlet: Betreff als Titel, die Mail als `.eml` oder Link als Arbeitsmittel
  (`tasqlet add --link`).
- **HatRack** wechselt ganze Arbeitskontexte (Makro-Ebene). Tasqlet ist die Mikro-Ebene: nächster
  Schritt, Checkliste, Gedanken. Eine Kopplung über `tags` spielt vorerst keine Rolle.
- **KI-Assistenten** prüfen und ergänzen Aufgaben über den MCP-Server (3.3), entscheiden aber nicht,
  was dran ist.
- **Sprint-Tray** zeigt den Team-Sprint. Tasqlet nimmt ein Ticket als Chip auf und zerlegt es
  persönlich.

## 11. Click-Dummy

`docs/click-dummy/index.html` zeigt das Zielbild: eine Datei ohne Build, im Browser öffnen. Er ist
**Vorlage, kein Bauplan**. Maßgeblich ist dieses Konzept. Wo der Dummy davon abweicht oder etwas offen
lässt, wird nachgefragt. Gestrichelte Markierungen im Dummy stammen aus dem Review und sind inzwischen
Teil dieses Konzepts.
