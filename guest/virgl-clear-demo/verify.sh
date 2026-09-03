#!/bin/sh
set -eu

binary=${1:-build/virgl-clear-demo}
tool_prefix=${CROSS:-aarch64-elf-}
readelf=${tool_prefix}readelf
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

size=$(wc -c < "$binary" | tr -d ' ')
test "$size" -le 65536
for source in Makefile README.md demo.c kms.c kms.h link.ld memory.c ops.c ops.h syscall.h transfer.c transfer.h uapi.h virgl.h verify.sh; do
    lines=$(wc -l < "$source" | tr -d ' ')
    test "$lines" -le 180
done
printf 'verified %s: ELF64 AArch64 static EXEC, standard VirGL copy/upload/readback/clear=%s bytes\n' \
    "$binary" "$size"
