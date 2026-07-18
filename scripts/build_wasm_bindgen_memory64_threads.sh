#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REVISION="ddd322514d87a4b21342b7ab9a9d70796fc60576"
REPOSITORY="https://github.com/wasm-bindgen/wasm-bindgen"
PATCH_FILE="$ROOT_DIR/patches/wasm-bindgen-memory64-threads.patch"
PATCH_ID="$(shasum -a 256 "$PATCH_FILE" | awk '{print $1}')"
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
BUILD_ROOT="${WASM_BINDGEN_BUILD_ROOT:-$ROOT_DIR/.artifacts/tool-build/wasm-bindgen-memory64-threads}"
SOURCE_DIR="$BUILD_ROOT/source-$REVISION-$PATCH_ID"
TARGET_DIR="$BUILD_ROOT/target-$REVISION-$PATCH_ID"
INSTALL_ROOT="${WASM_BINDGEN_INSTALL_ROOT:-$ROOT_DIR/.artifacts/tools/wasm-bindgen-memory64-threads}"
OUTPUT_BIN="$INSTALL_ROOT/bin/wasm-bindgen"

find_cached_checkout() {
    local candidate
    local head
    local checkouts="$CARGO_HOME_DIR/git/checkouts"

    [[ -d "$checkouts" ]] || return 1
    while IFS= read -r candidate; do
        head="$(git -C "$candidate" rev-parse HEAD 2>/dev/null || true)"
        if [[ "$head" == "$REVISION" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done < <(find "$checkouts" -mindepth 2 -maxdepth 2 -type d -print)
    return 1
}

prepare_source() {
    local cached
    local head

    if [[ -e "$SOURCE_DIR/.git" ]]; then
        head="$(git -C "$SOURCE_DIR" rev-parse HEAD 2>/dev/null || true)"
        [[ "$head" == "$REVISION" ]] || {
            echo "unexpected revision in $SOURCE_DIR: $head" >&2
            exit 1
        }
        return
    fi
    [[ ! -e "$SOURCE_DIR" ]] || {
        echo "build source exists but is not a Git checkout: $SOURCE_DIR" >&2
        exit 1
    }

    mkdir -p "$(dirname "$SOURCE_DIR")"
    if cached="$(find_cached_checkout)"; then
        echo "Reusing cached wasm-bindgen checkout: $cached"
        git clone --quiet --no-hardlinks "$cached" "$SOURCE_DIR"
        git -C "$SOURCE_DIR" checkout --quiet --detach "$REVISION"
    else
        echo "Fetching wasm-bindgen revision $REVISION"
        git init --quiet "$SOURCE_DIR"
        git -C "$SOURCE_DIR" remote add origin "$REPOSITORY"
        git -C "$SOURCE_DIR" fetch --quiet --depth 1 origin "$REVISION"
        git -C "$SOURCE_DIR" checkout --quiet --detach FETCH_HEAD
    fi
}

apply_patch_once() {
    if git -C "$SOURCE_DIR" apply --reverse --check "$PATCH_FILE" 2>/dev/null; then
        echo "Reusing patched wasm-bindgen source."
        return
    fi
    git -C "$SOURCE_DIR" diff --quiet
    git -C "$SOURCE_DIR" diff --cached --quiet
    git -C "$SOURCE_DIR" apply --check "$PATCH_FILE"
    git -C "$SOURCE_DIR" apply "$PATCH_FILE"
}

prepare_source
apply_patch_once
mkdir -p "$TARGET_DIR" "$INSTALL_ROOT"

CARGO_TARGET_DIR="$TARGET_DIR" \
    cargo install \
    --path "$SOURCE_DIR/crates/cli" \
    --root "$INSTALL_ROOT" \
    --bin wasm-bindgen \
    --force \
    --locked \
    --no-track

version="$("$OUTPUT_BIN" --version)"
[[ "$version" == "wasm-bindgen 0.2.122" ]] || {
    echo "unexpected patched CLI version: $version" >&2
    exit 1
}
echo "Patched wasm-bindgen CLI: $OUTPUT_BIN"
