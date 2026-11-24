# Click-Dummy: das Zielbild von Tasqlet

Abgestimmter Entwurf vom 2026-11-24. Er zeigt, wie Tasqlet aussieht und bedient wird. Er ist die
**Vorlage** für `docs/PLAN.md`, kein Bauplan: Maßgeblich ist `docs/KONZEPT.md`.

## Ansehen

```sh
xdg-open docs/click-dummy/index.html          # oder
python3 -m http.server 5393 -d docs/click-dummy   # dann http://localhost:5393
```

Veröffentlichte Fassung (privat): https://claude.ai/artifact/58zDEeLvuz41HvxW7DYCth

## Aufbau

Eine Datei ohne Build und ohne Abhängigkeiten (nur die Schrift Inter von Google Fonts).

- Links der Desktop mit Panel und Tray-Symbol (Tooltip beim Überfahren, Menü per Rechtsklick), darin
  das Popover.
- Rechts eine Spalte, die nicht zum Produkt gehört: Hinweise zum Ausprobieren, Simulation von Hotkey,
  Markierung und Zwischenablage, Farbthemen (Palette, beide Achsen, Vorschläge).
- Im Skript: `items` (Beispieldaten), Domänenfunktionen (`start`, `finish`, `toNext`, `moveBy`,
  `setState`, `delegate`), Ansichten (`cardHTML`, `rowHTML`, `ideaHTML`, `splitHTML` …), Ereignisse.
  Die Domänenfunktionen entsprechen den Übergängen in `KONZEPT.md` 2.4 und eignen sich als Vorlage für
  die Tests in `tasqlet-core`.
- Im CSS: alle Paletten (`.pop[data-tq=…]`) und die Achsen (`.pop[data-focus=…]`,
  `.pop[data-chrome=…]`). Diese Werte sind die Vorlage für die mitgelieferten Themen.

## Was der Dummy nicht tut

Er speichert nichts, öffnet nichts (statt `xdg-open` erscheint eine Meldung), kennt kein Archiv auf
der Platte und keinen echten Hotkey.
