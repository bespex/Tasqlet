#!/usr/bin/env bash
# Start Tasqlet as it is in the working tree.
#
#   ./run.sh [ARGS...]            with hot reload: frontend changes appear immediately,
#                                 Rust changes rebuild and restart the app
#   ./run.sh --release [ARGS...]  optimized build without packaging, then start it
#
# ARGS go to tasqlet. TASQLET_DATA_DIR / TASQLET_CONFIG_DIR start a separate instance.

set -euo pipefail

repo="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

release=false
case "${1:-}" in
  -h | --help)
    sed -n '2,9s/^# \{0,1\}//p' "$0"
    exit 0
    ;;
  --release)
    release=true
    shift
    ;;
esac

cd "$repo"

for tool in npm cargo; do
  command -v "$tool" >/dev/null || { echo "run.sh: $tool not found" >&2; exit 1; }
done

# One Tasqlet per data folder: a second start would only hand over to the running one
if [[ -z "${TASQLET_DATA_DIR:-}" ]] && pids="$(pgrep -x tasqlet)"; then
  echo "run.sh: Tasqlet is already running (PID ${pids//$'\n'/, }); this start would only show its window." >&2
  echo "        Quit it first, or start a separate instance:" >&2
  echo "        TASQLET_DATA_DIR=.scratch/data TASQLET_CONFIG_DIR=.scratch/config ./run.sh" >&2
  exit 1
fi

# Install or refresh dependencies only when package-lock.json is newer than the last install
if [[ ! -d node_modules || package-lock.json -nt node_modules/.package-lock.json ]]; then
  npm install --no-audit --no-fund
fi

if $release; then
  npm run --silent tauri -- build --no-bundle
  # Where cargo builds to: target/ here, unless configured otherwise (CARGO_TARGET_DIR, build.target-dir)
  target="$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
  exec "${target:-$repo/target}/release/tasqlet" "$@"
fi

# The dev server needs its fixed port; a second instance would fail with an unclear error
if (exec 3<>/dev/tcp/127.0.0.1/1430) 2>/dev/null; then
  echo "run.sh: port 1430 is in use - is another ./run.sh still running?" >&2
  exit 1
fi

# First `--` ends the tauri options, second `--` ends the cargo options; the rest goes to tasqlet
exec npx tauri dev -- -- "$@"
