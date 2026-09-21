# F03.3.2.2.3.6.3 evidence

Revision: 6463d13c1fbd5171bb97ec9eb92e6d0e2cb4123d
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_vertex_array_object_lifecycle_raw_inventory.json`
(`eac4887475fd7fd0ffec8a109d12c9dc999ca4aed87a41cc32aed943b98588c8`)
Profile: GLES 3.2 raw vertex-array-object lifecycle literals only; promotion disabled

Task ID and date: F03.3.2.2.3.6.3, 2026-09-22.

Tested code commit: `6463d13c1fbd5171bb97ec9eb92e6d0e2cb4123d`, parent
`fec93f5eb08a5b8e7642e35c81e29251377527fc`; patch SHA-256:
`baa1d00dae5ecda2c07b5bad5fc914400a858bb6a3fb6bca54226589a2553020`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has four ordered unprefixed names, each stored with its `gl` C
prefix: `GenVertexArrays`, `DeleteVertexArrays`, `BindVertexArray`, and
`IsVertexArray`. It binds `vertex-array` at source order 22. The raw-entry hash
is `3558a8edb746ac0b2c3211cb8a48844e6294059213e53adcc80308dec729daf5`;
the inventory hash is
`666a54c440e7a7bf6d9970e7cf3e7781fdd6bebf6a62c03ce2617db329dd3885`.

The source fence permits only pp.294–295, §10.4. Its primary locator is derived
as p.294/§10.4 and must match both authority admission and cache receipt. On
p.295, the running 10.5 header is not the boundary: the extractor ends only at
the later semantic heading `10.5 Drawing Commands Using Vertex Arrays`, thereby
excluding `DrawArraysOneInstance`. A fixed absolute extractor allowlist plus a
sterile child `PATH` rejects the tested ambient-PATH substitute; it does not
attest a privileged replacement race of an allowlisted binary.

Reproduce from repository root:

```sh
make graphics-gles-vertex-array-object-lifecycle-test
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/03-vertex-array-object-lifecycle-commands/gles_vertex_array_object_lifecycle_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: direct focused tests passed 7/7 in 95.537 s; the Make target
passed 7/7 in 100.594 s; the CLI passed `4 bounded GLES vertex-array-object
forms; source-only`. The target passed 7/7 in 89.004 s within full `make test`.
The full suite passed 1,127 Rust tests (3 ignored) and 338 web tests. Source
limits passed 6/6; diff and roadmap checks passed.

Negative checks reject missing, reordered, or cross-family forms; p.295 heading
drift and leaked draw declarations; wrong `boolean` return form; normalizer,
family, route, profile, authority-locator, or cache-locator drift; semantic
promotion; duplicate, nonfinite, stale, or symlinked artifacts; unavailable
ESSL/registry substitutes; private-loader/cache substitution; ambient module
aliases; and an ambient fake `pdftotext`. The artifact is self-hashed and raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no object existence or
binding effect, array state, draw, vertex processing, transform feedback,
guest/browser execution, support, conformance, certification, or performance
claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `6463d13c` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 6463d13c --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.6.4.
