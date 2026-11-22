#!/usr/bin/env bash
# Pack a relocatable prebuilt SDK archive (tar.gz or zip) from an install tree.
#
# Usage:
#   scripts/package_sdk_archive.sh <version> <platform> <arch> <staging> <out> [variant]
#
#   platform: linux | windows | macos
#   arch:     amd64 | arm64 | x64   (recorded in the caller-chosen output name)
#   staging:  usr/{bin,lib,include} (deb/pkg tree) or a flat {bin,lib,include} tree
#   variant:  main (default) | ros2-humble | ros2-jazzy
#
# The archive root is the SDK prefix (no usr/ layer). Protobuf and Abseil
# headers, libraries, tools, and CMake configs are copied from PROTOBUF_PREFIX
# (default /usr/local). On Linux, non-system shared libraries needed by the SDK
# are copied in with relative RPATHs. ROS libraries stay in the system install.
#
# Env:
#   PROTOBUF_PREFIX   install prefix of protobuf 35.1 (default /usr/local)
#   EXTRA_LIB_DIRS    colon-separated directories searched for NEEDED libraries
set -euo pipefail

VERSION="${1:?version}"
PLATFORM="${2:?platform}"
ARCH="${3:?arch}"
STAGING="${4:?staging}"
OUT="${5:?out archive}"
VARIANT="${6:-main}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROTOBUF_PREFIX="${PROTOBUF_PREFIX:-/usr/local}"

case "$PLATFORM" in
  linux|windows|macos) ;;
  *) echo "error: platform must be linux, windows, or macos (got '$PLATFORM')" >&2; exit 1 ;;
esac
case "$VARIANT" in
  main|ros2-humble|ros2-jazzy) ;;
  *) echo "error: variant must be main, ros2-humble, or ros2-jazzy (got '$VARIANT')" >&2; exit 1 ;;
esac
case "$PLATFORM" in
  windows)
    [[ "$OUT" == *.zip ]] || { echo "error: windows archive must be a .zip (got '$OUT')" >&2; exit 1; }
    ;;
  *)
    [[ "$OUT" == *.tar.gz ]] || { echo "error: $PLATFORM archive must be a .tar.gz (got '$OUT')" >&2; exit 1; }
    ;;
esac
if [[ "$VARIANT" == ros2-* && "$PLATFORM" != linux ]]; then
  echo "error: ROS 2 SDK archives are Linux only" >&2
  exit 1
fi

STAGING="$(cd "$STAGING" && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "staging SDK tree from $STAGING"
cp -a "$STAGING/." "$WORK/"
if [[ -d "$WORK/usr/lib" || -d "$WORK/usr/bin" || -d "$WORK/usr/include" ]]; then
  mkdir -p "$WORK/bin" "$WORK/lib" "$WORK/include"
  if [[ -d "$WORK/usr/bin" ]]; then cp -a "$WORK/usr/bin/." "$WORK/bin/"; fi
  if [[ -d "$WORK/usr/lib" ]]; then cp -a "$WORK/usr/lib/." "$WORK/lib/"; fi
  if [[ -d "$WORK/usr/include" ]]; then cp -a "$WORK/usr/include/." "$WORK/include/"; fi
  rm -rf "$WORK/usr"
fi
mkdir -p "$WORK/bin" "$WORK/lib" "$WORK/include" "$WORK/lib/pkgconfig" "$WORK/lib/cmake"

copy_dir_into() {
  local src="$1" dest="$2"
  [[ -d "$src" ]] || return 0
  mkdir -p "$dest"
  cp -a "$src/." "$dest/"
}

copy_named() {
  local src_dir="$1" pattern="$2" dest="$3"
  [[ -d "$src_dir" ]] || return 0
  mkdir -p "$dest"
  find "$src_dir" -maxdepth 1 -name "$pattern" -exec cp -a {} "$dest/" \;
}

echo "bundling protobuf/abseil from $PROTOBUF_PREFIX"
if [[ ! -d "$PROTOBUF_PREFIX" ]]; then
  echo "error: PROTOBUF_PREFIX does not exist: $PROTOBUF_PREFIX" >&2
  exit 1
fi
copy_dir_into "$PROTOBUF_PREFIX/include/google" "$WORK/include/google"
copy_dir_into "$PROTOBUF_PREFIX/include/absl" "$WORK/include/absl"
if [[ -d "$PROTOBUF_PREFIX/include" ]]; then
  find "$PROTOBUF_PREFIX/include" -maxdepth 1 -name 'utf8*' -exec cp -a {} "$WORK/include/" \;
fi
for pkg in protobuf absl utf8_range; do
  copy_dir_into "$PROTOBUF_PREFIX/lib/cmake/$pkg" "$WORK/lib/cmake/$pkg"
  copy_dir_into "$PROTOBUF_PREFIX/lib64/cmake/$pkg" "$WORK/lib/cmake/$pkg"
done
copy_named "$PROTOBUF_PREFIX/lib/pkgconfig" '*.pc' "$WORK/lib/pkgconfig"
copy_named "$PROTOBUF_PREFIX/lib64/pkgconfig" '*.pc' "$WORK/lib/pkgconfig"

case "$PLATFORM" in
  windows)
    copy_named "$PROTOBUF_PREFIX/bin" '*.dll' "$WORK/bin"
    copy_named "$PROTOBUF_PREFIX/lib" '*.dll' "$WORK/bin"
    copy_named "$PROTOBUF_PREFIX/lib" '*.lib' "$WORK/lib"
    ;;
  linux|macos)
    # The upstream CMake export checks all its targets, including protoc and
    # libraries unused by apps that only link protobuf::libprotobuf.
    for libdir in "$PROTOBUF_PREFIX/lib" "$PROTOBUF_PREFIX/lib64"; do
      for pattern in 'libprotobuf*' 'libprotoc*' 'libupb*' 'libabsl*' 'libutf8*'; do
        copy_named "$libdir" "$pattern" "$WORK/lib"
      done
    done
    ;;
esac
copy_named "$PROTOBUF_PREFIX/bin" 'protoc*' "$WORK/bin"

if [[ ! -f "$WORK/lib/cmake/protobuf/protobuf-config.cmake" ]]; then
  echo "error: $PROTOBUF_PREFIX has no lib/cmake/protobuf/protobuf-config.cmake" >&2
  find "$PROTOBUF_PREFIX" -name 'protobuf-config.cmake' -print >&2 || true
  exit 1
fi
if [[ ! -f "$WORK/lib/cmake/absl/abslConfig.cmake" && ! -f "$WORK/lib/cmake/absl/absl-config.cmake" ]]; then
  echo "error: $PROTOBUF_PREFIX has no lib/cmake/absl/abslConfig.cmake" >&2
  exit 1
fi

# Drop absolute prefixes in pkg-config files so they follow the archive.
if [[ -d "$WORK/lib/pkgconfig" ]]; then
  while IFS= read -r pc; do
    sed -i.bak -e 's|^prefix=.*|prefix=${pcfiledir}/../..|' "$pc"
    rm -f "$pc.bak"
  done < <(find "$WORK/lib/pkgconfig" -type f -name '*.pc')
fi

echo "writing relocatable CMake package"
"$ROOT/scripts/stage_robot_bus_cmake_config.sh" "$WORK/lib/cmake/robot_bus" "$VERSION"
cp -a "$ROOT/bindings/cpp/cmake/FetchRobotBus.cmake" "$WORK/lib/cmake/robot_bus/FetchRobotBus.cmake"

assert_relocatable_cmake() {
  local f
  while IFS= read -r f; do
    # Quoted absolute paths only. "${_IMPORT_PREFIX}/..." and "${PACKAGE_PREFIX_DIR}/..." are relocatable.
    if grep -E 'IMPORTED_LOCATION.*("(/|[A-Za-z]:[/\\]))' "$f" | grep -v '\${' | grep -q .; then
      echo "error: $f contains an absolute IMPORTED_LOCATION" >&2
      grep -n 'IMPORTED_LOCATION' "$f" >&2 || true
      exit 1
    fi
  done < <(find "$WORK/lib/cmake" -type f -name '*.cmake')
}
assert_relocatable_cmake

is_elf() {
  local magic
  magic=$(od -An -t x1 -N 4 "$1" 2>/dev/null | tr -d ' \n')
  [[ "$magic" == "7f454c46" ]]
}

is_macho() {
  local magic
  magic=$(od -An -t x1 -N 4 "$1" 2>/dev/null | tr -d ' \n')
  case "$magic" in
    cffaedfe|cefaedfe|cafebabe|feedfacf|bebafeca) return 0 ;;
  esac
  return 1
}

skip_system_soname() {
  case "$1" in
    libc.so*|libm.so*|libdl.so*|librt.so*|libpthread.so*|libresolv.so*|libgcc_s.so*|libstdc++.so*|ld-linux*|linux-vdso*|libutil.so*|libcrypt.so*|libnsl.so*|libBrokenLocale.so*)
      return 0
      ;;
  esac
  return 1
}

locate_needed() {
  local name="$1" dir cached
  if [[ -e "$WORK/lib/$name" ]]; then
    echo "$WORK/lib/$name"
    return 0
  fi
  cached=$(ldconfig -p 2>/dev/null | awk -v n="$name" '$1 == n { print $NF; exit }' || true)
  if [[ -n "$cached" && -e "$cached" ]]; then
    echo "$cached"
    return 0
  fi
  local search=()
  local extra
  IFS=':' read -r -a search <<< "${EXTRA_LIB_DIRS:-}"
  search+=("$PROTOBUF_PREFIX/lib" "$PROTOBUF_PREFIX/lib64")
  search+=(
    /usr/local/lib /usr/local/lib64
    /usr/lib /usr/lib64 /lib /lib64
    /usr/lib/x86_64-linux-gnu /lib/x86_64-linux-gnu
    /usr/lib/aarch64-linux-gnu /lib/aarch64-linux-gnu
  )
  for dir in "${search[@]}"; do
    [[ -n "$dir" && -e "$dir/$name" ]] || continue
    echo "$dir/$name"
    return 0
  done
  return 1
}

bundle_linux_deps() {
  command -v readelf >/dev/null 2>&1 || { echo "error: readelf is required" >&2; exit 1; }
  command -v patchelf >/dev/null 2>&1 || { echo "error: patchelf is required" >&2; exit 1; }
  local changed=1 guard=0 f name src real base needed rpath ros_dir
  local ros_lib_dirs=()
  if [[ "$VARIANT" == ros2-* ]]; then
    local ros_lib="/opt/ros/${VARIANT#ros2-}/lib"
    ros_lib_dirs=("$ros_lib" "$ros_lib"/*/)
  fi
  while [[ "$changed" -eq 1 ]]; do
    changed=0
    guard=$((guard + 1))
    if [[ "$guard" -gt 30 ]]; then
      echo "error: shared library dependency walk did not converge" >&2
      exit 1
    fi
    while IFS= read -r f; do
      if [[ -L "$f" ]]; then
        continue
      fi
      if ! is_elf "$f"; then
        continue
      fi
      needed=$(readelf -d "$f" | sed -n 's/.*NEEDED.*\[\(.*\)\].*/\1/p')
      for name in $needed; do
        if skip_system_soname "$name"; then
          continue
        fi
        if [[ -e "$WORK/lib/$name" ]]; then
          continue
        fi
        for ros_dir in "${ros_lib_dirs[@]}"; do
          if [[ -e "$ros_dir/$name" ]]; then
            continue 2
          fi
        done
        if ! src=$(locate_needed "$name"); then
          echo "error: $f needs $name, not found (PROTOBUF_PREFIX=$PROTOBUF_PREFIX)" >&2
          exit 1
        fi
        real=$(readlink -f "$src")
        base=$(basename "$real")
        echo "bundling $name from $real"
        cp -a "$real" "$WORK/lib/$base"
        if [[ "$base" != "$name" ]]; then
          ln -sfn "$base" "$WORK/lib/$name"
        fi
        changed=1
      done
    done < <(find "$WORK/lib" "$WORK/bin" -type f)
  done
  while IFS= read -r f; do
    if [[ -L "$f" ]]; then
      continue
    fi
    if ! is_elf "$f"; then
      continue
    fi
    if [[ "$(basename "$f")" == "librobot_bus.so" ]]; then
      patchelf --set-soname librobot_bus.so "$f"
    fi
    case "$f" in
      "$WORK/bin/"*) rpath='$ORIGIN/../lib' ;;
      *) rpath='$ORIGIN' ;;
    esac
    patchelf --set-rpath "$rpath" "$f"
  done < <(find "$WORK/lib" "$WORK/bin" -type f)
}

bundle_macos_deps() {
  command -v otool >/dev/null 2>&1 || { echo "error: otool is required" >&2; exit 1; }
  command -v install_name_tool >/dev/null 2>&1 || { echo "error: install_name_tool is required" >&2; exit 1; }
  local changed=1 guard=0 f dep depbase src real rpath
  while [[ "$changed" -eq 1 ]]; do
    changed=0
    guard=$((guard + 1))
    if [[ "$guard" -gt 30 ]]; then
      echo "error: dylib dependency walk did not converge" >&2
      exit 1
    fi
    while IFS= read -r f; do
      if [[ -L "$f" ]]; then
        continue
      fi
      if ! is_macho "$f"; then
        continue
      fi
      while IFS= read -r dep; do
        if [[ -z "$dep" ]]; then
          continue
        fi
        case "$dep" in
          /usr/lib/*|/System/*|@rpath/*|@loader_path/*|@executable_path/*) continue ;;
        esac
        depbase=$(basename "$dep")
        if [[ -e "$WORK/lib/$depbase" ]]; then
          continue
        fi
        if [[ ! -e "$dep" ]]; then
          echo "error: $f needs $dep, not found" >&2
          exit 1
        fi
        src="$dep"
        if [[ -L "$src" ]]; then
          real=$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$src")
        else
          real="$src"
        fi
        echo "bundling $depbase from $real"
        cp -a "$real" "$WORK/lib/$(basename "$real")"
        if [[ "$(basename "$real")" != "$depbase" ]]; then
          ln -sfn "$(basename "$real")" "$WORK/lib/$depbase"
        fi
        changed=1
      done < <(otool -L "$f" | awk 'NR>1 { print $1 }')
    done < <(find "$WORK/lib" "$WORK/bin" -type f)
  done
  while IFS= read -r f; do
    if [[ -L "$f" ]]; then
      continue
    fi
    if ! is_macho "$f"; then
      continue
    fi
    case "$f" in
      "$WORK/bin/"*) rpath='@loader_path/../lib' ;;
      *)
        rpath='@loader_path'
        install_name_tool -id "@rpath/$(basename "$f")" "$f"
        ;;
    esac
    while IFS= read -r dep; do
      if [[ -z "$dep" ]]; then
        continue
      fi
      case "$dep" in
        /usr/lib/*|/System/*|@rpath/*|@loader_path/*|@executable_path/*) continue ;;
      esac
      install_name_tool -change "$dep" "@rpath/$(basename "$dep")" "$f"
    done < <(otool -L "$f" | awk 'NR>1 { print $1 }')
    if ! otool -l "$f" | grep -F "path $rpath (offset " >/dev/null; then
      install_name_tool -add_rpath "$rpath" "$f"
    fi
    if command -v codesign >/dev/null 2>&1; then
      codesign --force --sign - "$f" >/dev/null || echo "warning: codesign failed for $f" >&2
    fi
  done < <(find "$WORK/lib" "$WORK/bin" -type f)
}

case "$PLATFORM" in
  linux) bundle_linux_deps ;;
  macos) bundle_macos_deps ;;
esac

case "$PLATFORM" in
  linux) [[ -e "$WORK/lib/librobot_bus.so" ]] || { echo "error: missing lib/librobot_bus.so" >&2; exit 1; } ;;
  macos) [[ -e "$WORK/lib/librobot_bus.dylib" ]] || { echo "error: missing lib/librobot_bus.dylib" >&2; exit 1; } ;;
  windows)
    for file in bin/robot_bus_c.dll bin/robot_bus_msgs.dll lib/robot_bus.lib lib/robot_bus_msgs.lib; do
      [[ -f "$WORK/$file" ]] || { echo "error: missing $file" >&2; exit 1; }
    done
    ;;
esac
if [[ "$VARIANT" == ros2-* ]]; then
  if [[ -e "$WORK/opt/ros" || -d "$WORK/opt/ros" ]]; then
    echo "error: refusing to ship vendored /opt/ros" >&2
    exit 1
  fi
  if find "$WORK" -name 'librcl.so*' | grep -q .; then
    echo "error: refusing to ship librcl" >&2
    exit 1
  fi
  [[ -e "$WORK/lib/librobot_bus_ros2_bridge.so" ]] || {
    echo "error: missing lib/librobot_bus_ros2_bridge.so" >&2
    exit 1
  }
fi
if [[ -f "$WORK/bin/robot_bus_broker" ]]; then
  chmod 755 "$WORK/bin/robot_bus_broker"
fi

out_parent="$(dirname -- "$OUT")"
mkdir -p -- "$out_parent"
if ! out_parent="$(cd -- "$out_parent" && pwd)"; then
  echo "error: cannot resolve output directory for '$OUT'" >&2
  exit 1
fi
OUT_ABS="${out_parent}/$(basename -- "$OUT")"
rm -f "$OUT_ABS" "${OUT_ABS}.sha256"
echo "archiving $OUT_ABS ($PLATFORM $ARCH $VARIANT)"
(
  cd "$WORK"
  if [[ "$PLATFORM" == windows ]]; then
    cmake -E tar cf "$OUT_ABS" --format=zip -- bin include lib
  else
    cmake -E tar czf "$OUT_ABS" -- bin include lib
  fi
)

if command -v sha256sum >/dev/null 2>&1; then
  HASH="$(sha256sum "$OUT_ABS" | awk '{print $1}')"
else
  HASH="$(shasum -a 256 "$OUT_ABS" | awk '{print $1}')"
fi
printf '%s  %s\n' "$HASH" "$(basename "$OUT_ABS")" >"${OUT_ABS}.sha256"
echo "wrote $OUT_ABS"
echo "wrote ${OUT_ABS}.sha256 ($HASH)"
