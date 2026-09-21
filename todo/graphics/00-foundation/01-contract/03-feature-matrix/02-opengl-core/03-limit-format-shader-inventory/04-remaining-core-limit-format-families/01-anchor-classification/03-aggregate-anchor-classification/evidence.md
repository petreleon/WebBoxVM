# F03.2.3.4.1.3 evidence

Revision: a5ed365e514124020b9d9fcd0d6eb3c3d52dc64a
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_closed_anchor_classification.json` (`bbceb0d5173e765250e3ca83b2e838086defb58944ce69d914089e5b2d578c42`)
Profile: OpenGL 4.6 core closed source-review classification only

Task ID and date: F03.2.3.4.1.3, 2026-09-21.

Tested commit and patch: parent `9f7911eddb3f76531f91fa5312b754246dbc2a15`;
the tested tree became `a5ed365e`; its patch SHA-256 is
`c2eea0e2e6bfc09367a4f8750c5afe01678299a390f100bebc5a5b121d5554b6`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The aggregate binds the exact 159 final-table and 38 chapter-local candidates once,
in child source order, with immutable PDF locators and a fixed route/reason. Its
four self-hashed fragments contain 197 candidates: 162 `eligible-unreviewed`, 28
`route-to-state`, three `shader-unadmitted`, four `extension-unadmitted`, and zero
`covered`/`out-of-domain`. It routes review work to F03.2.2.3.2, F03.2.3.2, and
F03.2.3.4.2–F03.2.3.4.5.4; it does not promote any candidate into a fact.

Reproduce from repository root:

```sh
make graphics-opengl-aggregate-anchor-classification-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/03-aggregate-anchor-classification/opengl_closed_anchor_classification.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 8/8; the CLI passed `197 closed source-only
candidates; 162 eligible-unreviewed`; `make test` passed, including 1,127 Rust unit
tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject changed profile/count/claim/Matrix/CTS values, rehashed child
receipt substitutions, duplicate/unanchored/promoted rows, wildcard policy routes, a
rehashed persisted candidate fragment, symlinked/forged fixed helpers, and ambient
preloaded helper aliases in a fresh interpreter. The receipt compares every fragment
and child receipt exactly against a fresh rendering from the sealed cache.

Raw captures: no external capture was retained; the sealed cache, immutable source
locators, child receipts, aggregate receipt, and fragment hashes reproduce this
finite routing result. No native/browser run, software fallback, or performance
protocol applies.

Decision and limits: this is a source-review routing receipt only. It asserts no API
property, limit/format behavior, shader/extension authority, implementation/support,
conformance, certification, guest/browser behavior, or performance result.

Commit/push verification: `a5ed365e` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
a5ed365e --limit 10` returned no runs.

Next ready tasks: F03.2.3.4.2, F03.2.3.4.3, F03.2.3.4.4.1, and F03.2.3.4.5.1.
