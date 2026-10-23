#!/usr/bin/env bash
# Собирает оригинальное ядро Xray-core (Go) в libtytxray.so для трёх ABI через Android NDK.
# Нужны: Go (версия из xraycore/go.mod) и ANDROID_NDK_HOME.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME не задан}"
api="${ANDROID_MIN_API:-21}"
case "$(uname -s)" in
  Linux)  host=linux-x86_64 ;;
  Darwin) host=darwin-x86_64 ;;
  *) echo "неподдерживаемая ОС сборки: $(uname -s)" >&2; exit 1 ;;
esac
bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$host/bin"
out="$root/app/src/main/jniLibs"

build() { # abi goarch cc [goarm]
  local abi="$1" goarch="$2" cc="$3" goarm="${4:-}"
  echo "==> Xray-core для $abi"
  mkdir -p "$out/$abi"
  (
    cd "$root/xraycore"
    export CGO_ENABLED=1 GOOS=android GOARCH="$goarch" CC="$bin/$cc"
    [ -n "$goarm" ] && export GOARM="$goarm"
    go build -trimpath -buildvcs=false -buildmode=c-shared \
      -ldflags="-s -w -buildid=" -o "$out/$abi/libtytxray.so" ./libtyt
    rm -f "$out/$abi/libtytxray.h"
  )
}

build arm64-v8a   arm64 "aarch64-linux-android${api}-clang"
build armeabi-v7a arm   "armv7a-linux-androideabi${api}-clang" 7
build x86_64      amd64 "x86_64-linux-android${api}-clang"
ls -la "$out"/*/libtytxray.so
