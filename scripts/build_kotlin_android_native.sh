#!/usr/bin/env bash
# Deprecated alias — use scripts/build_android_native.sh
exec "$(cd "$(dirname "$0")" && pwd)/build_android_native.sh" "$@"
