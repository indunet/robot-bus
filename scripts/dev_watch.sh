#!/usr/bin/env bash
# Watch Rust + console sources → rebuild embedded UI (if needed) + restart broker.
# Open http://127.0.0.1:15770 after it comes up.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

run_once() {
  local stamp="$root/assets/console/.watch-stamp"
  local need_console=0
  if [[ ! -f "$stamp" ]] || [[ ! -f "$root/assets/console/index.html" ]]; then
    need_console=1
  elif find \
      console/app console/components console/lib console/public \
      console/package.json console/next.config.mjs console/tsconfig.json \
      bindings/typescript/src \
      -type f -newer "$stamp" 2>/dev/null | grep -q .; then
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

  echo "Starting broker → http://127.0.0.1:15770"
  exec cargo run --bin robot_bus_broker
}

if [[ "${1:-}" == "--once" ]]; then
  export RUST_LOG="${RUST_LOG:-info}"
  run_once
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
exec cargo watch \
  --clear \
  -d 1 \
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
