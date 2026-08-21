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

# Honor PROTOC= (full path). On Windows GHA, Git Bash PATH may not see a
# MSYS-style install unless we prepend the compiler's directory.
if [[ -n "${PROTOC:-}" ]]; then
  proto_dir="$(dirname "$PROTOC")"
  if command -v cygpath >/dev/null 2>&1; then
    proto_dir="$(cygpath -u "$proto_dir" 2>/dev/null || echo "$proto_dir")"
  fi
  export PATH="$proto_dir:$PATH"
fi
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
