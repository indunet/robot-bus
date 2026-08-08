#!/usr/bin/env bash
# Watch Rust + console sources → rebuild embedded UI (if needed) + restart broker.
# Open http://127.0.0.1:15570 after it comes up.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

# Free the console/gRPC port so a restart can bind (leftover brokers otherwise
# leave cargo-watch spam "[Running …]" while the old process keeps serving).
stop_broker_listeners() {
  local pids
  pids="$(lsof -nP -iTCP:15570 -sTCP:LISTEN -t 2>/dev/null || true)"
  if [[ -n "${pids}" ]]; then
    echo "Stopping previous listener(s) on :15570 → ${pids//$'\n'/ }"
    # shellcheck disable=SC2086
    kill ${pids} 2>/dev/null || true
    sleep 0.3
    pids="$(lsof -nP -iTCP:15570 -sTCP:LISTEN -t 2>/dev/null || true)"
    if [[ -n "${pids}" ]]; then
      # shellcheck disable=SC2086
      kill -9 ${pids} 2>/dev/null || true
    fi
  fi
}

console_sources_newer_than() {
  local stamp="$1"
  # -print -quit: stop at first hit (fast). `|| true` keeps pipefail/set -e calm
  # when nothing is newer.
  find \
    console/app console/components console/lib console/public \
    console/package.json console/next.config.mjs console/tsconfig.json \
    bindings/typescript/src \
    -type f -newer "$stamp" -print -quit 2>/dev/null | grep -q .
}

run_once() {
  local stamp="$root/assets/console/.watch-stamp"
  local need_console=0

  stop_broker_listeners

  if [[ ! -f "$stamp" ]] || [[ ! -f "$root/assets/console/index.html" ]]; then
    need_console=1
  elif console_sources_newer_than "$stamp"; then
    need_console=1
  fi

  if [[ "$need_console" -eq 1 ]]; then
    echo "Building console → assets/console/ ..."
    just console
    mkdir -p "$root/assets/console"
    touch "$stamp"
  else
    echo "Console assets up to date; skipping just console."
  fi

  echo "Starting broker → http://127.0.0.1:15570"
  # Do not `exec`: cargo-watch needs a killable process group; with `exec` the
  # broker can survive a flaky restart and keep serving stale assets.
  cargo run --bin robot_bus_broker
}

if [[ "${1:-}" == "--once" ]]; then
  export RUST_LOG="${RUST_LOG:-info}"
  run_once
  exit $?
fi

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "cargo-watch not found; installing (once)..."
  cargo install cargo-watch --locked
fi

export RUST_LOG="${RUST_LOG:-info}"

echo "Watching src/ + console sources (Ctrl+C to stop)..."
# Do NOT use --watch-when-idle: that flag ignores FS events while `cargo run`
# is alive, so Rust/TS edits never restart the broker. Ignore build outputs
# instead (below) so `just console` does not loop.
# Debounce 2s so bursty editor saves don't kill a mid-flight console build.
exec cargo watch \
  --clear \
  -d 2 \
  -w src \
  -w console/app \
  -w console/components \
  -w console/lib \
  -w console/public \
  -w console/package.json \
  -w console/next.config.mjs \
  -w console/tsconfig.json \
  -w bindings/typescript/src \
  -w Cargo.toml \
  -i '**/node_modules/**' \
  -i '**/.next/**' \
  -i '**/out/**' \
  -i '**/tsconfig.tsbuildinfo' \
  -i '**/next-env.d.ts' \
  -i 'assets/console/**' \
  -i 'bindings/typescript/dist/**' \
  -i 'bindings/typescript/generated/**' \
  -s "bash \"$root/scripts/dev_watch.sh\" --once"
