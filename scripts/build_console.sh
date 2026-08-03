#!/usr/bin/env bash
# Build console static export into assets/console for rust-embed.
#
# Console imports the in-repo TypeScript SDK (`robot-bus`). Both
# `bindings/typescript/generated/` and `bindings/typescript/dist/` are
# gitignored, so codegen + `build:ts` must run before `next build`.
#
# Requires: protoc 35.1 on PATH, Node/npm, pnpm.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
ts="$root/bindings/typescript"

if ! command -v protoc >/dev/null 2>&1; then
  echo "error: protoc is required to generate bindings/typescript/generated" >&2
  exit 1
fi
if ! command -v pnpm >/dev/null 2>&1; then
  echo "error: pnpm is required to build console/" >&2
  exit 1
fi

(
  cd "$ts"
  if [[ -f package-lock.json ]]; then
    npm ci
  else
    npm install
  fi
)

python3 "$root/scripts/generate_typescript_msgs.py"

(
  cd "$ts"
  npm run build:ts
)

(
  cd "$root/console"
  pnpm install --frozen-lockfile
  pnpm build
)

"$root/scripts/sync_console_assets.sh"
