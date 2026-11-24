# Entscheidungen

Getroffene Entscheidungen mit Grund, neueste unten. Offene Fragen stehen in `PLAN.md` („Entscheidungen
vor dem Start“).

| # | Datum | Entscheidung | Grund |
|---|---|---|---|
| D1 | 2026-11-24 | Keine Zeitmessung, keine Laufzeit-Anzeige | Organisieren, nicht überwachen. Wechsel zwischen Aufgaben ist normaler Alltag |
| D2 | 2026-11-24 | Datum nur notiert, nie eingefärbt, sortiert oder erinnert | Kein Druck durch Fristen |
| D3 | 2026-11-24 | „↩ zuletzt dran“ statt „⏸ Pausiert“, Pause als Kaffeetasse | Keine Protokoll-Sprache. Pause ist eine bewusste Pause, keine Unterbrechung |
| D4 | 2026-11-24 | Nach „Erledigt“ ist #2 „Bereit“, kein Auto-Start | Was als Nächstes dran ist, entscheidet der Mensch |
| D5 | 2026-11-24 | `!` für Datum, `@` für Personen, `#` frei für Kontext | `@fr` gegen `@Frank` war mehrdeutig |
| D6 | 2026-11-24 | Drei wichtigste Arbeitsmittel sichtbar, `…` für alle. Ziffer `Alt+n` fest am Arbeitsmittel | Überblick. Verlässliche Tasten trotz Umsortierung |
| D7 | 2026-11-24 | Speicherung als eine JSON-Datei plus Jahresarchiv, Markdown nur als Export | Umsortieren und laufende App vertragen sich schlecht mit einer Datei je Eintrag |
| D8 | 2026-11-24 | Tauri 2, `tasqlet-core`, `ksni`, Svelte 5 | Wiederverwendung aus Maillet und CodeDeck, Gestaltungsfreiheit |
| D9 | 2026-11-24 | Kein Drop aufs Tray-Symbol | StatusNotifierItem kennt kein Drop |
| D10 | 2026-11-24 | Hotkey in MVP 1 über `tasqlet --toggle`, Portal erst in MVP 3 | Funktioniert sofort auf jedem Desktop, auch unter Wayland |
| D11 | 2026-11-24 | Themen aus Palette und zwei Achsen (aktuelle Aufgabe, Bedienelemente), Kontrastprüfung beim Laden | Freiheit in der Gestaltung ohne unlesbare Themen |
| D12 | 2026-11-24 | Frontend mit Svelte 5 | Leicht und schnell, im Ökosystem schon bei Pictorix |
| D13 | 2026-11-24 | Code, Kommentare, Commit-Messages Englisch, Doku Deutsch (vorerst) | Code bleibt für alle lesbar, Konzept und Plan in der Arbeitssprache |
| D14 | 2026-11-24 | Commits bündeln, wo sinnvoll: eine Einheit je abgeschlossenem Schritt oder Teilschritt | Lesbare Historie statt vieler Kleinst-Commits |
| D15 | 2026-11-24 | Keine KI-Hinweise in Commits und Pull Requests, KI nicht als Co-Autor | Soll in GitHub nicht erkennbar sein. Lokaler `commit-msg`-Hook prüft |
| D16 | 2026-11-24 | Keine eingebaute Messung in der App. Einmalige Messung (Cinnamon/X11, Release-Build): Popover nach dem Zeigen in ~20 ms gezeichnet; Speicher ~350 MiB RSS inkl. WebKit-Prozesse (überzeichnet, da geteilte Bibliotheken mehrfach zählen) | Wie Maillet und HatRack: Messwerkzeuge gehören nicht ins Produkt. Ziel „unter 100 ms“ ist klar erfüllt |
| D17 | 2026-11-24 | Mindestversion Rust 1.90 (wie Maillet) | Mit 1.85 hielt der Resolver `tauri` auf 2.11, das npm-Paket ist 2.12; `tauri build` bricht bei ungleichen Versionen ab |
| D18 | 2026-11-24 | Übergänge im Core, Lücken im Konzept so gefüllt: Undo über Schnappschüsse des ganzen Stores (50 Schritte, nur im Speicher). „Als Nächstes“ gilt nicht für #1. „Zurück in Fokus“ vergisst `delegated_to`/`delegated_at`. Erledigtes lässt sich wieder in eine Liste holen (`done_at` fällt weg). Ohne Titel und ohne Notiz bleibt der Titel leer, die Oberfläche zeigt einen Platzhalter. `last_active` bleibt auch auf einer „Bereit“-Karte stehen. Delegieren ohne Namen wird abgelehnt | Schnappschüsse machen Undo für jede Aktion exakt, ohne Umkehr-Logik je Übergang. Die übrigen Punkte folgen den Regeln aus 2.1 und 2.3 am direktesten |
| D19 | 2026-11-24 | Bevorzugter Stil: Vorschlag „Leitstand“ (Palette Fjord, aktuelle Aufgabe betont, Bedienelemente abgesetzt) | Wunsch des Nutzers nach Sichtung des Dummys |
| D20 | 2026-11-24 | Arbeitskarte ohne Kopfzeile (Entwurf K2): runder `✓` rechts oben (bei „Bereit“ `▶`), darunter beim Überfahren kleine runde Knöpfe für Pause, Delegieren, Mehr. Klick auf den Titel = Ausarbeiten. Status und notiertes Datum in einer Fußleiste | Der Kopf des Dummys reihte Pause, Erledigt, Delegieren und Ausarbeiten ohne Ordnung. So sind die Bedienelemente rechts klar gruppiert, und die Fußleiste hat Platz für spätere Funktionen |
