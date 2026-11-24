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

Cargo-Workspace im Root (`crates/tasqlet-core`, `src-tauri`), Frontend im Root (`src/`, Vite auf
Port 1430). Voraussetzungen: Rust stable, Node.js 22, `libwebkit2gtk-4.1-dev`, `librsvg2-dev`.

```sh
npm run dev                      # UI im Browser mit Mock-Daten (http://localhost:1430)
npm run tauri dev                # Desktop-App
npm run check                    # svelte-check + prettier --check
npm test                         # vitest
npm run format                   # prettier --write
cargo fmt --all --check
mkdir -p dist && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Dieselben Schritte laufen in der CI (`.github/workflows/ci.yml`). Vor einem Commit müssen alle grün sein.

## Karte

| Pfad | Zuständig für |
|---|---|
| `crates/tasqlet-core/src/lib.rs` | Domäne (noch leer bis auf `SCHEMA_VERSION`) |
| `src-tauri/src/lib.rs` | Tauri-Builder, Commands (`schema_version`) |
| `src-tauri/tauri.conf.json` | Fenster `popover` (470 px, rahmenlos), Bundle `.deb`/AppImage, Identifier `eu.spex.tasqlet` |
| `src-tauri/icons/icon.svg` | Quelle der App-Symbole (`npx tauri icon src-tauri/icons/icon.svg -o src-tauri/icons`) |
| `src/lib/backend.ts` | alle Aufrufe an Rust, mit Mock (`inTauri`, `mock`) |
| `src/app.css` | Rollen-Variablen `--tq-*` (vollständig ab Schritt 1.5) |
