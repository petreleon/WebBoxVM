#!/usr/bin/env bash
set -euo pipefail

base="${DEBIAN_ARM64_ISO_BASE:-https://cdimage.debian.org/debian-cd/current/arm64/iso-cd}"
out="${DEBIAN_ARM64_ISO_OUT:-.artifacts/debian-arm64-netinst.iso}"
file="${DEBIAN_ARM64_ISO_FILE:-}"
dry_run="${DRY_RUN:-0}"

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "missing required tool: $1" >&2
        exit 127
    fi
}

need curl
need grep
need awk
need shasum

base="${base%/}"
mkdir -p "$(dirname "$out")"

tmp="$(mktemp -d)"
cleanup() {
    rm -rf "$tmp"
}
trap cleanup EXIT

index="$tmp/index.html"
sums="$tmp/SHA256SUMS"

curl -fsSL "$base/" -o "$index"

if [[ -z "$file" ]]; then
    file="$(
        grep -Eo 'debian-[^"/]+-arm64-netinst\.iso' "$index" \
            | awk '!seen[$0]++ { print; exit }' \
            || true
    )"
fi

if [[ -z "$file" ]]; then
    echo "could not discover a Debian arm64 netinst ISO at $base" >&2
    exit 1
fi

curl -fsSL "$base/SHA256SUMS" -o "$sums"
expected="$(
    awk -v file="$file" '
        $2 == file || $2 == "*" file {
            print $1
            exit
        }
    ' "$sums"
)"

if [[ -z "$expected" ]]; then
    echo "could not find checksum for $file in $base/SHA256SUMS" >&2
    exit 1
fi

echo "Debian ARM64 ISO: $file"
echo "Source: $base/$file"
echo "Output: $out"

if [[ "$dry_run" == "1" ]]; then
    echo "Dry run only; not downloading."
    exit 0
fi

if [[ -f "$out" ]]; then
    actual="$(shasum -a 256 "$out" | awk '{ print $1 }')"
    if [[ "$actual" == "$expected" ]]; then
        echo "Already downloaded and checksum matches."
        ls -lh "$out"
        exit 0
    fi
    echo "Existing output checksum does not match; replacing it."
fi

download="$tmp/$file"
curl -fL --progress-bar "$base/$file" -o "$download.part"
mv "$download.part" "$download"

(
    cd "$tmp"
    printf '%s  %s\n' "$expected" "$file" | shasum -a 256 -c -
)

mv "$download" "$out"
chmod 0644 "$out"
printf '%s\n' "$file" > "$out.name"
printf '%s  %s\n' "$expected" "$(basename "$out")" > "$out.sha256"
ls -lh "$out"
