# Tasqlet

Focus and notes in your Linux tray: capture a thought or a link in seconds, see what is next at a
glance, and open the ticket, mail or file of the current task with one click.

One task is in focus. Everything else stays calm: a short queue of what is urgent, small things for
in between, and ideas that grow over days. Local data, no cloud, and no time tracking.

> **Status:** early development. Nothing to install yet. The target design is the click dummy in
> [`docs/click-dummy/`](docs/click-dummy/index.html); the plan is in [`docs/PLAN.md`](docs/PLAN.md)
> (German).

## Development

Requirements: Rust (stable), Node.js 22, and the Tauri system libraries
(`libwebkit2gtk-4.1-dev`, `librsvg2-dev`).

```sh
npm install
npm run dev          # UI in the browser with mock data, http://localhost:1430
npm run tauri dev    # the desktop app
npm run check        # svelte-check and prettier
npm test             # frontend tests
cargo test --workspace
```

Part of [be spex](https://bespex.github.io) – apps for the way you work.

## License

MIT
