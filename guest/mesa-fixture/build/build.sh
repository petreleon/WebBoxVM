#!/bin/sh
set -eu
recipe=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH= cd -- "$recipe/../../.." && pwd)
input=$root/.artifacts/graphics/i01-mesa-image
output=$input/build
archive=$input/mesa-06f9e28304d5d3f109c33535c1c25b9df5769af2.tar.gz
attempt=${I01_BUILD_ATTEMPT:-01}
case "$attempt" in [0-9][0-9]) ;; *) echo 'Build attempt must have two digits' >&2; exit 1 ;; esac
test -f "$archive"
test ! -d "$output/mesa-destdir"
python3 "$recipe/verify.py" --input-only
builder=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["image_id"])' "$output/manifests/builder.json")
docker run --platform linux/arm64 --network=none --cpus=4 --memory=8g \
    --name "webboxvm-i01-mesa-build-06f9e283-$attempt" -e "I01_BUILD_ATTEMPT=$attempt" \
    --tmpfs /work:rw,exec,nosuid,nodev,size=3g,mode=755 \
    --tmpfs /source:rw,nosuid,nodev,size=768m,mode=755 \
    -v "$output:/output" -v "$recipe:/recipe:ro" -v "$archive:/input/archive.tar.gz:ro" \
    "$builder" \
    python3 /recipe/compile.py
