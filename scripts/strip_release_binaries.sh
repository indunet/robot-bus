#!/usr/bin/env bash
# Usage: bash scripts/strip_release_binaries.sh <symbols-dir> <staged-binary>...
set -euo pipefail

symbols_dir="${1:?symbols directory}"
shift
if [[ "$#" -eq 0 ]]; then
  echo "error: expected at least one staged binary" >&2
  exit 1
fi
mkdir -p "$symbols_dir"
platform="$(uname -s)"

for binary in "$@"; do
  name="${binary##*/}"
  case "$platform" in
    Linux)
      debug_file="$symbols_dir/$name.debug"
      objcopy --only-keep-debug "$binary" "$debug_file"
      objcopy --strip-unneeded --add-gnu-debuglink="$debug_file" "$binary"
      ;;
    Darwin)
      cp -p "$binary" "$symbols_dir/$name.unstripped"
      xcrun strip -S -x "$binary"
      ;;
    *)
      echo "error: stripping release binaries is unsupported on $platform" >&2
      exit 1
      ;;
  esac
done
