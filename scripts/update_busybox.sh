#!/usr/bin/env bash
set -euo pipefail

image="${1:-busybox:1.37.0-musl}"
platform="${BUSYBOX_PLATFORM:-linux/arm64}"
out="${BUSYBOX_OUT:-.artifacts/busybox-aarch64}"

mkdir -p "$(dirname "$out")"

cid="$(docker create --platform="$platform" "$image")"
cleanup() {
    docker rm "$cid" >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker cp "$cid:/bin/busybox" "$out"
chmod 0644 "$out"
file "$out"
ls -lh "$out"
