#!/usr/bin/env bash
# Copy Next.js static export (console/out) into assets/console for rust-embed.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
src="$root/console/out"
dst="$root/assets/console"

if [[ ! -f "$src/index.html" ]]; then
  echo "error: $src/index.html not found — run: ./scripts/build_console.sh" >&2
  exit 1
fi

rm -rf "$dst"
mkdir -p "$dst"
cp -R "$src"/. "$dst"/
echo "synced console assets → assets/console/ ($(find "$dst" -type f | wc -l | tr -d ' ') files)"
