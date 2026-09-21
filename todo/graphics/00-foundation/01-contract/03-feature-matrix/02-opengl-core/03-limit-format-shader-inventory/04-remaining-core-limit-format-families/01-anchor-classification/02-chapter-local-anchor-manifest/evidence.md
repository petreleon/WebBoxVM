# F03.2.3.4.1.2 evidence

Revision: bf3b3ec96f8433061ed0df80f342c0b2c3ff6514
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_chapter_local_anchor_manifest.json` (`d81803d7811059338c9ce5656ffbe3f8ecd8bbb79158c814397a1442571e3b38`)
Profile: OpenGL 4.6 core chapter-local candidate manifest only; unclassified

Task ID and date: F03.2.3.4.1.2, 2026-09-21.

Tested commit and patch: parent `e884fa453eb46b52e1ba21f8388bdcbc653347a0`;
the tested staged tree became `bf3b3ec9`; its patch SHA-256 is
`1202ec0b016e212dae97d5397f20f9493340227871624009c4ae749d201579e6`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, 3,003,752 bytes,
SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The manifest seals 38 candidates in seven self-hashed fragments: Table 6.5;
Tables 8.2–8.27, 9.1–9.3, 10.3–10.6, 18.2, 18.4, and 22.2; plus §22.3. The PDF
contains no Table 22.3, so that final roadmap item is a section-heading candidate,
not an invented table. Every entry remains `unclassified`, with caption/heading-only
scope, source order, zero semantic facts, zero Matrix rows, zero CTS runs, and false
claims.

Reproduce from repository root:

```sh
make graphics-opengl-chapter-local-anchor-manifest-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/03-limit-format-shader-inventory/04-remaining-core-limit-format-families/01-anchor-classification/02-chapter-local-anchor-manifest/opengl_chapter_local_anchor_manifest.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `38 chapter-local
candidates; unclassified-only`; full `make test` passed, including 1,127 Rust unit
tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject a missing, duplicate, reordered, wildcard, stale,
cross-profile, or promoted entry; missing/wrong PDF anchors; a rehashed fragment;
and a missing fixed dependency. No external capture was retained; the sealed cache,
manifest, and fragment hashes reproduce this finite review universe.

Decision and limits: this is not a classification, API fact, limit/format result,
shader/extension authority, implementation/support, conformance, certification,
guest/browser behavior, or performance result.

Commit/push verification: `bf3b3ec9` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
bf3b3ec9 --limit 10` returned no runs.

Next ready task: F03.2.3.4.1.3.
