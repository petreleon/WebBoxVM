#!/bin/sh
set -eu

binary=${1:-build/webgpu-demo}
tool_prefix=${CROSS:-aarch64-elf-}
readelf=${tool_prefix}readelf
objcopy=${tool_prefix}objcopy
nm=${tool_prefix}nm

test -f "$binary"
test -x "$binary"
"$readelf" -h "$binary" | grep -q 'Class:.*ELF64'
"$readelf" -h "$binary" | grep -q 'Type:.*EXEC'
"$readelf" -h "$binary" | grep -q 'Machine:.*AArch64'
! "$readelf" -l "$binary" | grep -q 'INTERP'
! "$readelf" -d "$binary" 2>&1 | grep -q '(NEEDED)'
test -z "$("$nm" -u "$binary")"
"$nm" "$binary" | grep -q ' T _start$'

packet_tmp=$(mktemp "${TMPDIR:-/tmp}/webgpu-demo-packet.XXXXXX")
trap 'rm -f "$packet_tmp"' EXIT HUP INT TERM
"$objcopy" --dump-section .wbg3="$packet_tmp" "$binary"
test "$(wc -c < "$packet_tmp" | tr -d ' ')" = 408
header=$(od -An -tx1 -N16 "$packet_tmp" | tr -d ' \n')
test "$header" = 57424733010000000100000001000000

size=$(wc -c < "$binary" | tr -d ' ')
test "$size" -le 65536
for source in .gitignore Makefile README.md demo.c link.ld packet.c packet.h syscall.h uapi.h verify.sh; do
    lines=$(wc -l < "$source" | tr -d ' ')
    test "$lines" -le 180
done
printf 'verified %s: ELF64 AArch64 static EXEC, WBG3 packet=408 bytes, file=%s bytes\n' \
    "$binary" "$size"
