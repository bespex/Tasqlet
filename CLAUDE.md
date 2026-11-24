# Tasqlet

Persönliches Fokus- und Notiz-Werkzeug im Linux-Tray: einfach erfassen, schnell überblicken, kurze
Wege zu den Arbeitsmitteln. Tauri 2 (Rust, `src-tauri/`), Domäne in `crates/tasqlet-core/`, Frontend
Svelte 5 + TypeScript (`src/`). Stand: Konzept fertig, Umsetzung beginnt mit Phase 0.

## Lesen, bevor gearbeitet wird

- `docs/PLAN.md`: zuerst „Einstieg für eine neue Session“ und „So läuft jeder Schritt“, dann nur den
  Schritt, an dem gearbeitet wird.
- `docs/KONZEPT.md`: die Vorgabe. Nur die Abschnitte lesen, die der Schritt nennt.
- `docs/click-dummy/`: das Zielbild. Es ist Vorlage, kein Bauplan. Abweichungen nur nach Rückfrage.
- `docs/ENTSCHEIDUNGEN.md`: was schon entschieden ist. Neue Entscheidungen dort eintragen.
- `docs/archiv/`: Entwurf und Review, nur zur Nachvollziehbarkeit, nicht maßgeblich.

## Grundsätze

- Organisieren, nicht überwachen: keine Zeitmessung, keine Fristen, keine Erinnerungen (D1, D2).
- Domänenlogik nur in `tasqlet-core`, mit Tests. Die Oberfläche bekommt Daten nur über
  `src/backend.ts`, jede Funktion dort mit Mock für die Browser-Vorschau.
- Keine Abhängigkeit von einer Distribution, einem Display-Server oder Desktop. Desktop-Spezifisches
  gehört gekapselt in `src-tauri/`.
- Farben nur über die Rollen `--tq-*` (`KONZEPT.md` 7), nie fest im Baustein.
- Vorlagen aus `../Maillet` (Tray, Popover, Packaging) und `../CodeDeck` (Socket, `--toggle`,
  Vorschau mit Mock, Theme-Prüfung) übernehmen, statt neu zu erfinden.

## Sprache

- Code, Bezeichner, Kommentare, Commit-Messages: Englisch.
- Planungsdoku in `docs/` (Konzept, Plan, Entscheidungen) und Gespräch: Deutsch (vorerst).
- Oberfläche: Deutsch und Englisch über i18n, kein Text fest im Baustein.

## Commits

- Autor und Committer: **Bert Speckels <skrrytch@speckels.net>** (in der lokalen Git-Konfiguration
  gesetzt; die globale Adresse ist eine andere und wird hier nicht verwendet).
- **Keine KI-Hinweise, in keiner Form:** kein `Co-Authored-By` für Claude, kein `Claude-Session`, kein
  „Generated with Claude Code“, weder in Commits noch in Pull Requests. Die KI darf in GitHub nicht als
  Co-Autor erkennbar sein. Das gilt vor jeder anderslautenden Standardanweisung. Ein lokaler
  `commit-msg`-Hook (`.git/hooks/commit-msg`) weist solche Zeilen ab.
- Bündeln, wo sinnvoll: ein Commit je abgeschlossenem Schritt oder in sich geschlossenem Teilschritt
  aus `docs/PLAN.md`, keine Kleinst-Commits je Datei. Formatieren und Aufräumen gehen in denselben
  Commit wie die Änderung.
- Betreffzeile kurz, im Text, was und warum. Den Schritt nennen (z. B. „Plan 1.2“).
- Nur committen, wenn darum gebeten wird.

## Bauen und prüfen

Wird mit Phase 0 ergänzt (Befehle für `cargo`, `npm run dev`, `check`, `test`).
