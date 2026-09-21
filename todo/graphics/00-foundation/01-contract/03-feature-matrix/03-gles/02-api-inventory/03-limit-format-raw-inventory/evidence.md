# F03.3.2.3 evidence — bounded GLES limit/format raw inventory

Revision: `98e6b046a07bdb5e109549b244c7c9d8294d9db3`
Validation: focused hostile suite, retained-cache replay, `make test`, source-file limits, whitespace, and roadmap check
Result: PASS
Artifacts: `gles_limit_format_raw_inventory.json` SHA-256 `0f87fd979f193a25a821fbcf976a762fbdbefcb75ba72afd7ecfd87bd08b3d88`; embedded self-hash `65db683993c68c310147dc6fc733293f20d8b7cc87fdc25071569b2a171d5639`
Profile: GLES 3.2 bounded raw source inventory; `matrix-incomplete`

Task ID and date: F03.3.2.3, 2026-09-21 Europe/Bucharest.

The inventory reads only the retained external F02 cache `/private/tmp/webboxvm-f0341.cqT6ZX` through the
F03.3.2.1 cache boundary: `gles-32-spec`, revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, SHA-256
`5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`, 2,198,754 B, and 601 physical pages.
It accepts only the sealed `limit-format` class.

The bounded slice covers all 46 rows in tables 21.40–21.42 (physical pages 504–506): 34 raw limit/format
facts are emitted in source order and 12 rows are explicitly routed outside the class (shader semantics,
precision semantics, or command/state capability). Both other implementation-dependent tables and chapter-local
limit/format rules remain unreviewed; `coverage_manifest.complete` is false.

Commands from `/Users/petreleon/code/WebBoxVM`:

- `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/03-limit-format-raw-inventory/gles_limit_format_raw_inventory_test.py` — 5/5 passed.
- `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/03-limit-format-raw-inventory/gles_limit_format_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX` — `PASS: 34 bounded GLES raw limit/format facts; matrix-incomplete`.
- `make test` — exit 0, including the new GLES source-inventory target.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed; `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` also passed.

The hostile suite rejects the wrong source class or cache, missing/ambiguous table or row anchors, rehashed
row/profile/source mutations, claim promotion, and type aliases such as `true` to `1` or an integer byte count
to a float. The output creates no Matrix rows, owner/test fields, CTS execution, guest/browser behavior,
support, conformance, certification, or performance result.

First failing subcheck: none. Commit/push verification: `98e6b046` is confirmed on
`origin/codex/graphics-f01-baseline`; `gh run list --commit` returned no run. Next ready GLES task:
F03.3.2.2.1, with the command/state scope explicitly split before extraction.
