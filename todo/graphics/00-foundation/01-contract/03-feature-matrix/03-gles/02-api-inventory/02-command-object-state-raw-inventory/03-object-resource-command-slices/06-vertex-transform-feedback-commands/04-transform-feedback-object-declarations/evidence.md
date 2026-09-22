# F03.3.2.2.3.6.4 evidence

Revision: 1ed89ee6c6ec1a2b7f1c315b21a4c70e561353e3
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_transform_feedback_object_raw_inventory.json`
(`642869e7d9ff35c01d00d235061d5bfc33c12d1f5e60a1d39a90fd1eae3fad76`)
Profile: GLES 3.2 raw transform-feedback-object literals only; promotion disabled

Task ID and date: F03.3.2.2.3.6.4, 2026-09-22.

Tested code commit: `1ed89ee6c6ec1a2b7f1c315b21a4c70e561353e3`, parent
`6d7b9dfed0d64425a48c8452725702adff5d5cea`; patch SHA-256:
`e2a70457608c0a8304695d29c8d8cd08ad7e0e848f325b4dfb593bb7f27db01a`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has four ordered unprefixed names, each stored with its `gl` C
prefix: `GenTransformFeedbacks`, `DeleteTransformFeedbacks`,
`IsTransformFeedback`, and `BindTransformFeedback`. It binds
`transform-feedback` at source order 26. The raw-entry hash is
`a5fa8bf34b1f561e795313203a420b189ef7e279e3ff0e58b79e24b6997e13e5`;
the inventory hash is
`e0e7b5576be6ded9c112d71b3c137bb0e6c3a706c004fdfbe80b454ed55e6347`.

The domain family correctly retains its canonical p.357/§12.2
`BeginTransformFeedback` anchor, but this leaf independently derives its
primary locator from p.354/§12.2.1. The source window begins only after the
exact p.354 heading `12.2.1 Transform Feedback Objects`, covers pp.354–356,
and requires the p.357 semantic heading `12.2.2 Transform Feedback Primitive
Capture`. Thus capture-control forms cannot enter this object slice. A fixed
absolute extractor allowlist plus sterile child `PATH` rejects the tested
ambient-PATH substitute; it does not attest a privileged replacement race of an
allowlisted binary.

Reproduce from repository root:

```sh
make graphics-gles-transform-feedback-object-test
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/04-transform-feedback-object-declarations/gles_transform_feedback_object_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 7/7 in 113.712 s; the CLI passed
`4 bounded GLES transform-feedback-object forms; source-only`. The target
passed 7/7 in 111.730 s within full `make test`. The full suite passed 1,127
Rust tests (3 ignored) and 338 web tests. Source limits passed 6/6; diff and
roadmap checks passed.

Negative checks reject missing, reordered, or cross-family forms; removed
`const` or changed `boolean` return forms; moving `GenTransformFeedbacks`
before the §12.2.1 heading; extension into p.357 capture; heading drift;
normalizer, family, route, profile, authority-locator, or cache-locator drift;
semantic promotion; duplicate, nonfinite, stale, or symlinked artifacts;
unavailable ESSL/registry substitutes; private-loader/cache substitution;
ambient module aliases; and an ambient fake `pdftotext`. The artifact is
self-hashed and raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no object, buffer, capture,
primitive-processing, state, draw, vertex-processing, guest/browser execution,
support, conformance, certification, or performance claim. It has no runtime
Matrix or CTS execution.

Commit/push verification: `1ed89ee6` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 1ed89ee6 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.6.5.
