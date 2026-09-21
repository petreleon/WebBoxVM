# F03.3.2.2.3.5.3 evidence

Revision: 140a2b5482ed5ca50f207cb454c093a2a5d93fac
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_framebuffer_attachment_status_command_raw_inventory.json`
(`e2dc7ff9199a59ddfb5bee623e44730b6badf29313d35f3ce06b27f9e07d63f8`)
Profile: GLES 3.2 raw framebuffer-attachment/status literals only; promotion disabled

Task ID and date: F03.3.2.2.3.5.3, 2026-09-21.

Tested code commit: `140a2b5482ed5ca50f207cb454c093a2a5d93fac`, parent
`24ab8e4c4a736bbaf42c4e8c7adab21d9e254438`; patch SHA-256:
`0e7924142247e3621dfe027c16670771a33878248697df2608b04e41e308d008`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has five ordered C names: `FramebufferRenderbuffer`,
`FramebufferTexture`, `FramebufferTexture2D`, `FramebufferTextureLayer`, and
`CheckFramebufferStatus`. The raw-entry hash is
`72f6588953b50ef3f308e124edd3ba448228912c03328cb608d914dcaa58a743`;
the whole inventory hash is
`e7e8f5b4e38e8592ac32fe8187ac36ed1c4f7f480779cfc911b7b4938cb53fee`.
The sealed source windows are p.256, pp.258–260, and p.269 under sections
9.2.7, 9.2.8, and 9.4.2. Pages 256 and 258 are scanned only after their fixed
headings; the p.267 heading is the witness for the p.269 `enum` declaration.

Reproduce from repository root:

```sh
make graphics-gles-framebuffer-attachment-status-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/03-framebuffer-attachment-status-commands/gles_framebuffer_attachment_status_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused direct tests passed 7/7 in 104.631 s; the Make target
passed 7/7 in 104.499 s; the CLI passed `5 bounded GLES framebuffer
attachment/status forms; source-only`. Full `make test` passed, including 1,127
Rust unit tests (3 ignored) and 338 web tests. Source limits passed 6/6; diff
and roadmap checks passed.

Negative checks reject omitted, reordered, truncated, return-type, or
cross-family forms; marker loss and pre-heading leakage on both fenced pages;
page, section, witness, normalizer, profile, domain, route, or family-order
drift; semantic promotion; malformed, nonfinite, or symlinked artifacts;
unavailable ESSL/registry substitutes; and private-loader or cache substitution.
The artifact is self-hashed and raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no attachment effect,
image/level/layer selection, completeness, status value, format, ESSL,
guest/browser execution, support, conformance, certification, or performance
claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `140a2b54` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 140a2b54 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.6.
