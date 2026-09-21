# F03.3.2.2.3.5.2 evidence

Revision: add97482ef758367d9a78826ac1ce427b590c288
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_renderbuffer_object_command_raw_inventory.json`
(`2629280b61d6d0746c3e2e30a332e0edca8512b88c2518ba266ab1763a94ff3e`)
Profile: GLES 3.2 raw renderbuffer-object literals only; promotion disabled

Task ID and date: F03.3.2.2.3.5.2, 2026-09-21.

Tested code commit: `add97482ef758367d9a78826ac1ce427b590c288`, parent
`aad9111582e08185620f1ae01adff31166232c6f`; patch SHA-256:
`1a1bc406248d99d21f8da847a4c69e47073dc50d9f8c9c893f24b2c742827e5d`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has seven ordered C names: `BindRenderbuffer`, `GenRenderbuffers`,
`DeleteRenderbuffers`, `IsRenderbuffer`, `RenderbufferStorageMultisample`,
`RenderbufferStorage`, and `GetRenderbufferParameteriv`. The raw-entry hash
is `dff86c12d61a611840c9386cd387e1d2a0ba232f91923b57baeb21b29911f6e1`;
the whole inventory hash is
`3d1b3c4f16c35bcccf8f11782f8c4ac2f65a3c118b6d264c81d02b83095253e9`.
The sealed source window covers pp.252–256 under sections 9.2.4 and 9.2.6.
It scans p.252 only after its renderbuffer heading, p.255 only before required
formats, and p.256 only before §9.2.7, excluding `FramebufferRenderbuffer`.
`IsRenderbuffer` retains its source `boolean` return form; both catalog and
cache source identity seal the profile exactly as `gles-3.2`.

Reproduce from repository root:

```sh
make graphics-gles-renderbuffer-object-storage-query-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/02-renderbuffer-object-storage-query-commands/gles_renderbuffer_object_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused direct tests passed 7/7 in 100.226 s; the Make target
passed 7/7 in 91.836 s; the CLI passed `7 bounded GLES renderbuffer-object
forms; source-only`. Full `make test` passed, including 1,127 Rust unit
tests (3 ignored) and 338 web tests. Source limits passed 6/6; diff and roadmap
checks passed.

Negative checks reject omitted/reordered forms; a framebuffer intrusion; all
three missing page markers; p.256 attachment intrusion; section, normalizer,
profile, domain, route, or family-order drift; semantic promotion; malformed,
nonfinite, or symlinked artifacts; unavailable ESSL/registry substitutes; and
private-loader or cache substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no renderbuffer storage,
multisampling, allocation, format, dimension, content, query-result, attachment,
ESSL, guest/browser execution, support, conformance, certification, or
performance claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `add97482` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit add97482 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.5.3.
