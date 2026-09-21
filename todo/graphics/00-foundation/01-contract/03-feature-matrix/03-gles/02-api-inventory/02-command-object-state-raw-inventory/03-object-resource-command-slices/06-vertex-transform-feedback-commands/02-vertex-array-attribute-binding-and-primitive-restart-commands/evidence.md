# F03.3.2.2.3.6.2 evidence

Revision: e43d2f20a5586b9d74de9a1748af011681f19ab0
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_vertex_array_binding_command_raw_inventory.json`
(`b1937fd47da8a4a153866f7a33eb8c7ff5f36585c9dbed721f185f89ca55d391`)
Profile: GLES 3.2 raw vertex-array binding and primitive-restart literals only; promotion disabled

Task ID and date: F03.3.2.2.3.6.2, 2026-09-22.

Tested code commit: `e43d2f20a5586b9d74de9a1748af011681f19ab0`, parent
`75e3146fbc86ee20d2a9fbe483c8ea66668b4f7d`; patch SHA-256:
`714bf29f9f9eae512ae4c6e450d752486d86f1c4738a239e5e57a9abf6ea2ed5`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has twelve ordered unprefixed names, each stored with its `gl` C
prefix: `VertexAttribFormat`, `VertexAttribIFormat`, `BindVertexBuffer`,
`VertexAttribBinding`, `VertexAttribPointer`, `VertexAttribIPointer`,
`EnableVertexAttribArray`, `DisableVertexAttribArray`, `VertexBindingDivisor`,
`VertexAttribDivisor`, `Enable`, and `Disable`. It binds `vertex-array` at
source order 22. The raw-entry hash is
`b01494278f8007b8427aa5b97bc4d9ddb4b13b17b03757df7329819c24f13cf8`;
the inventory hash is
`974b081c5b78d192cce0a6ec23cc8fe34331c310a33f22b82a3bd20d2742f04d`.

The source fence permits only pp.285–291: §10.3.1, §10.3.2, and §10.3.4.
It splits p.289 at §10.3.2, keeps p.290 before §10.3.3 and p.291 before
§10.3.5, and requires `PRIMITIVE_RESTART_FIXED_INDEX` beside generic
`Enable`/`Disable`. The primary locator is derived as p.285/§10.3.1 and must
match both authority admission and cache receipt. A fixed absolute extractor
allowlist plus sterile child `PATH` rejects the tested ambient-PATH substitute;
it does not attest a privileged replacement race of an allowlisted binary.

Reproduce from repository root:

```sh
make graphics-gles-vertex-array-binding-command-test
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/02-vertex-array-attribute-binding-and-primitive-restart-commands/gles_vertex_array_binding_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: direct focused tests passed 7/7 in 110.828 s; the Make target
passed 7/7 in 112.667 s; the CLI passed `12 bounded GLES vertex-array binding
forms; source-only`. The target passed 7/7 in 116.410 s within full `make test`.
The full suite passed 1,127 Rust tests (3 ignored) and 338 web tests. Source
limits passed 6/6; diff and roadmap checks passed.

Negative checks reject missing, reordered, or cross-family forms; §10.3.1/10.3.2
misrouting; page/heading/section-boundary/primitive-restart witness drift;
normalizer, family, route, profile, authority-locator, or cache-locator drift;
semantic promotion; duplicate, nonfinite, stale, or symlinked artifacts;
unavailable ESSL/registry substitutes; private-loader/cache substitution; ambient
module aliases; and an ambient fake `pdftotext`. The artifact is self-hashed and
raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no buffer association,
attribute state, primitive-restart effect, format, limit, draw, vertex processing,
transform feedback, guest/browser execution, support, conformance, certification,
or performance claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `e43d2f20` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit e43d2f20 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.6.3.
