# F03.3.2.2.3.4.1 evidence

Revision: 519de165920e82cf29fa1b5600e346c81a8e4268
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_texture_sampler_object_parameter_inventory.json`
(`a4c762571ebcf4ea1c52acc597f4cbda125061a49c495506beffb770ed29ce11`)
Profile: GLES 3.2 raw object/parameter forms only; promotion disabled

Task ID and date: F03.3.2.2.3.4.1, 2026-09-21.

Tested code commit: `519de165920e82cf29fa1b5600e346c81a8e4268`, parent
`777ac2f8e99fa7570e10c7e4bd8bd805e26457ed`; patch SHA-256:
`255bb68963eceaf0af23672b480de308c9f4cafb74719610810ca92ecf8d8bf9`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 14 ordered forms from physical pp.157–161: eight object
literals plus six names expanded from the three p.160 sampler-parameter
templates. Its raw-entry hash is
`c366ada8758e70cb3bdbf10e73211a18ae92b24a6d5b4f342d82aacd2500096e`.
It binds texture family order 17 and sampler family order 18. The source
spelling for `GenTextures` intentionally retains `;;`; p.158 and p.161 forms
are constrained to precede their next section headings. §8.3 sampler queries,
texture-unit state, values, state, sampling, lifetime, and semantics stay out.

Reproduce from repository root:

```sh
make graphics-gles-texture-sampler-object-parameter-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/01-texture-sampler-objects-parameters/gles_texture_sampler_object_parameter_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `14 bounded GLES
texture/sampler object forms; source-only`. Full `make test` passed, including
1,127 Rust unit tests (3 ignored) and 338 web tests. Source limits passed 6/6;
diff and roadmap checks passed.

Negative checks reject an altered `GenTextures` semicolon, missing/reordered
forms, source-page or heading-boundary drift, p.162 `GetSamplerParameter*`
intrusion, grammar order/count or expansion drift, stale domain order/route,
fabricated semantic fields, stale/duplicate/nonfinite/symlinked artifacts,
unavailable ESSL substitutes, and cache/private-loader substitution. The
artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal declaration,
physical page, section, source order, and source family. It creates no object
validity, parameter/state/sampling/lifetime behavior, texture format, image
contents, guest/browser execution, support, conformance, certification, or
performance claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `519de165` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 519de165 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.4.2.
