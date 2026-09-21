# F03.2.3.4.1.1 evidence

Revision: e50861976fb011e78a3a384a4ba73f8f8bb1c409
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_final_table_anchor_catalog.json` (`ba57c11bcdfd5e4440fa0c8f2e905359e7e5a9d3e4a2c7ddfae2f3135e1ba092`)
Profile: OpenGL 4.6 core final-table source classification only

Task ID and date: F03.2.3.4.1.1, 2026-09-21.

Tested commit and dirty diff hash: the staged tree tested before the code commit
became `e5086197`; parent `0702f339`; resulting patch SHA-256
`1b18f370c02274821f18c0674c0b566c55e46d3adb821ec12d11131fbc91df90`.

Source identity: OpenGL 4.6 core PDF, 851 physical pages, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`.
Guest image/build, browser/adapter/driver: not applicable to source classification.

Reproduction from repository root:

```sh
make graphics-opengl-final-table-anchor-catalog-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/01-final-table-anchors/opengl_final_table_anchor_catalog.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: 159 candidates—111 in Tables 23.56–23.66 and 48 in Tables
23.67–23.70/23.72–23.74; Table 23.71 is explicitly reviewed elsewhere. Focused
tests passed 6/6; the catalog CLI passed 159 classification-only candidates; `make
test` passed; source limits passed 6/6; diff and roadmap checks passed.

Routes: 124 eligible-unreviewed, 28 route-to-state, three shader-unadmitted, four
extension-unadmitted, and zero covered/out-of-domain. Negative tests reject altered
row labels, order, numeric sections, policy destinations, promoted entries, cache
absence, and table-wide fallbacks.

Raw captures: no external capture was retained; exact ordered PDF witnesses and
self-hashes in the committed catalog reproduce the result. No native/browser run,
software fallback, or performance protocol applies.

Decision and limits: the catalog routes review work only. It asserts no API limit,
format behavior, shader or extension support, implementation, conformance,
certification, guest/browser behavior, or performance.

Commit/push verification: `e5086197` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
e5086197 --limit 10` returned no runs.

Next ready task: F03.2.3.4.1.2.
