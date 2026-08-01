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

CANDLE="$(command -v candle.exe || command -v candle)"
LIGHT="$(command -v light.exe || command -v light)"

# Product.wxs always refs ComponentGroup Harvested. heat fills it with headers/libs;
# without heat, emit an empty group so candle/light still link.
if command -v heat.exe >/dev/null 2>&1 || command -v heat >/dev/null 2>&1; then
  HEAT="$(command -v heat.exe || command -v heat)"
  # Harvest include/ + lib/ only — bin/ is owned by ProductComponents (PATH + key files).
  HARVEST_ROOT="$WORK/harvest_root"
  mkdir -p "$HARVEST_ROOT"
  cp -a "$STAGE/include" "$HARVEST_ROOT/"
  cp -a "$STAGE/lib" "$HARVEST_ROOT/"
  "$HEAT" dir "$HARVEST_ROOT" -cg Harvested -gg -sfrag -srd -dr INSTALLFOLDER \
    -var var.StageDir -out "$WORK/harvest.wxs"
else
  cat >"$WORK/harvest.wxs" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Fragment>
    <ComponentGroup Id="Harvested" />
  </Fragment>
</Wix>
EOF
fi

"$CANDLE" -dProductVersion="$VERSION" -dStageDir="$STAGE" -out "$WORK/" "$WXS" "$WORK/harvest.wxs"
"$LIGHT" -out "$OUT_MSI" "$WORK"/*.wixobj

echo "wrote $OUT_MSI"
