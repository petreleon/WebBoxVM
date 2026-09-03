#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(dirname -- "$script_dir")
disk=${1:-"$repo_dir/output/webboxvm-final-install-compact.wbdisk"}
demo=${2:-"$repo_dir/guest/virgl-clear-demo/build/virgl-clear-demo"}

make -C "$repo_dir/guest/virgl-clear-demo"
exec cargo run --manifest-path "$repo_dir/Cargo.toml" -p emulator \
    --release --example virgl_guest_transport_smoke -- "$disk" "$demo"
