#!/usr/bin/env bash
# Build a Windows MSI from a staged install tree using WiX.
# Usage: scripts/package_msi.sh <version> <stage-dir> <out-msi>
# Requires candle.exe / light.exe on PATH (WiX Toolset).
set -euo pipefail

VERSION="${1:?version}"
STAGE="${2:?stage}"
OUT_MSI="${3:?out-msi}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WXS="$ROOT/bindings/cpp/packaging/msi/Product.wxs"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Prefer harvesting full tree when heat is available.
if command -v heat.exe >/dev/null 2>&1 || command -v heat >/dev/null 2>&1; then
  HEAT="$(command -v heat.exe || command -v heat)"
  CANDLE="$(command -v candle.exe || command -v candle)"
  LIGHT="$(command -v light.exe || command -v light)"
  "$HEAT" dir "$STAGE" -cg Harvested -gg -sfrag -srd -dr INSTALLFOLDER -var var.StageDir -out "$WORK/harvest.wxs"
  "$CANDLE" -dProductVersion="$VERSION" -dStageDir="$STAGE" -out "$WORK/" "$WXS" "$WORK/harvest.wxs"
  "$LIGHT" -out "$OUT_MSI" "$WORK"/*.wixobj
else
  CANDLE="$(command -v candle.exe || command -v candle)"
  LIGHT="$(command -v light.exe || command -v light)"
  "$CANDLE" -dProductVersion="$VERSION" -dStageDir="$STAGE" -out "$WORK/" "$WXS"
  "$LIGHT" -out "$OUT_MSI" "$WORK"/*.wixobj
fi

echo "wrote $OUT_MSI"
