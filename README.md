# Tasqlet

Focus and notes in your Linux tray: capture a thought or a link in seconds, see what is next at a
glance, and open the ticket, mail or file of the current task with one click.

One task is in focus. Everything else stays calm: a short queue of what is urgent, small things for
in between, and ideas that grow over days. Local data, no cloud, and no time tracking.

## Install

Ubuntu 24.04+, Linux Mint 22+, Debian 13+. From the [be spex APT repository](https://bespex.run/apt/)
(set it up once, updates then come with `apt upgrade`):

```sh
sudo apt install tasqlet
```

Or take the `.deb` or the AppImage from the [releases](https://github.com/bespex/Tasqlet/releases).
Start **Tasqlet** from the application menu: it lives in the tray. On X11 set a key to show and hide
it in the settings (gear icon); on Wayland bind `tasqlet --toggle` to a key of your desktop. Autostart
at login is in the settings too.

Drop files or links from the file manager or the browser onto an entry: they become its resources.
Pin the popover (pin icon) so it stays open while you drag.

### Command line

```sh
tasqlet add "Check the build !fri" --link https://ci.example.com/42   # end of the focus list
tasqlet add "Ask about the invoice @Anna"                             # delegated
tasqlet add "Rename the folder" --inter                               # also --now, --idea
tasqlet current [--json]          # the current task and its resources
tasqlet list [--state focus|intermezzo|idea|delegated|done] [--json]
tasqlet open [N]                  # resource N of the current task, or all
```

With Tasqlet running, the commands go to it (and show at once, with undo); otherwise they work on the
data files directly.

The plan and the concept are in [`docs/`](docs/) (German).

## Development

Requirements: Rust (stable), Node.js 22, and the Tauri system libraries
(`libwebkit2gtk-4.1-dev`, `librsvg2-dev`).

```sh
npm install
npm run dev          # UI in the browser with mock data, http://localhost:1430
./run.sh             # the desktop app with hot reload (--release: optimized build)
npm run check        # svelte-check and prettier
npm test             # frontend tests
cargo test --workspace
```

Part of [be spex](https://bespex.github.io) – apps for the way you work.

## License

MIT
