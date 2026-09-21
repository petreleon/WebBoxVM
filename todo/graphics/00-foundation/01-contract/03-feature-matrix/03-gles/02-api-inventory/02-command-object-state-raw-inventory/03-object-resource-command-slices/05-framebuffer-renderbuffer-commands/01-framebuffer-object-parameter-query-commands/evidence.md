# F03.3.2.2.3.5.1 evidence

Revision: f9d53b7dac5b5e528cc98fd945efdc584ea213e3
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_framebuffer_object_command_raw_inventory.json`
(`0063c9560284af59874d642ff9c0f320acaabfdb75937b62b3b8534f95ffaa32`)
Profile: GLES 3.2 raw framebuffer-object literals only; promotion disabled

Task ID and date: F03.3.2.2.3.5.1, 2026-09-21.

Tested code commit: `f9d53b7dac5b5e528cc98fd945efdc584ea213e3`, parent
`e1526422f3cc0ddc031b868d18fd07401ed9acad`; patch SHA-256:
`dd8e4ca57f5942e68dcf8ffe152e2d0fb6a2cc33fcc4211efa1b527b26cffa61`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set has seven ordered C names: `BindFramebuffer`, `GenFramebuffers`,
`DeleteFramebuffers`, `IsFramebuffer`, `FramebufferParameteri`,
`GetFramebufferParameteriv`, and `GetFramebufferAttachmentParameteriv`.
The raw-entry hash is
`b4c2b3947afba4e6f5604970c64f9ca7d898f3ad0196459d950a1f46ba2a9458`;
the whole inventory hash is
`cb0c8586f3e79653dd70ce909c5649086f9bbcbc215549d74d4c230d45134c66`.
The sealed source window covers pp.242, 244–245, and 248 under sections 9.2,
9.2.1, and 9.2.3. Page 245 is explicitly split before/after
`9.2.1 Framebuffer Object Parameters`; `IsFramebuffer` retains its source
`boolean` return form.

Reproduce from repository root:

```sh
make graphics-gles-framebuffer-object-parameter-query-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/05-framebuffer-renderbuffer-commands/01-framebuffer-object-parameter-query-commands/gles_framebuffer_object_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused direct tests passed 7/7 in 81.372 s; the Make target
passed 7/7 in 83.465 s; the CLI passed `7 bounded GLES framebuffer-object
forms; source-only`. Full `make test` passed, including 1,127 Rust unit
tests (3 ignored) and 338 web tests. Source limits passed 6/6; diff and roadmap
checks passed.

Negative checks reject omitted/reordered forms; a renderbuffer intrusion; source
page, section, heading-marker, normalizer, domain, route, or family-order drift;
crossing the p.245 section fence; semantic promotion; malformed, nonfinite, or
symlinked artifacts; unavailable ESSL/registry substitutes; and private-loader
or cache substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only literal spelling, formal declaration,
physical page, section, and source order. It creates no framebuffer binding or
parameter behavior, query result, attachment, completeness, format, limit,
ESSL, guest/browser execution, support, conformance, certification, or
performance claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `f9d53b7d` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit f9d53b7d --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.5.2.
