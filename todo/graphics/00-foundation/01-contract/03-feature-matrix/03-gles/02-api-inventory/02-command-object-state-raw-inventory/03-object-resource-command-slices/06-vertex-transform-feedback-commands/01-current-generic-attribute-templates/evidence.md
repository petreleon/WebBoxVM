# F03.3.2.2.3.6.1 evidence

Revision: 53ab6e42e297d85295ed1490822ef7c70fa0d204
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_current_vertex_attribute_template_inventory.json`
(`5a8fc242d64af900094649c79b20e57a5497439a83a3cca632edd4b398043e81`)
Profile: GLES 3.2 raw current-generic-attribute template expansions only; promotion disabled

Task ID and date: F03.3.2.2.3.6.1, 2026-09-22.

Tested code commit: `53ab6e42e297d85295ed1490822ef7c70fa0d204`, parent
`6ad5537b19e6fc1d58a443fc0c682c1cf1ce394b`; patch SHA-256:
`58e8ef7dbee923ff9627c5499cc043ed847e50acea2458b2d8af56a825e52efa`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set expands exactly four ordered p.283, §10.2.1 templates into twelve
ordered unprefixed names, each stored with its `gl` C prefix: `VertexAttrib1f`
through `VertexAttrib4f`, `VertexAttrib1fv` through `VertexAttrib4fv`,
`VertexAttribI4i`, `VertexAttribI4ui`, `VertexAttribI4iv`, and
`VertexAttribI4uiv`. It binds the `vertex-array`
family at source order 22 and grammar template orders 23–26. The raw-entry hash
is `37345f1f498973316468a72bc7671616171b3490288cb309c2d438c8fa8b84fd`;
the inventory hash is
`52641449ca512d23e79da4b61fda1cedcee5d1643fadd2cf83b6e4b99f40e36b`.

The validator derives the precise source locator from its sealed page and
section and requires the same locator in both authority admission and cache
receipt. It permits only fixed absolute `pdftotext` candidates with a sterile
child `PATH`; the hostile test places a fake extractor only on ambient `PATH`.
This closes the tested PATH-only substitution route. It does not attest an
allowlisted extractor binary against a privileged replacement race.

Reproduce from repository root:

```sh
make graphics-gles-current-vertex-attribute-template-test
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/01-current-generic-attribute-templates/gles_current_vertex_attribute_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: direct focused tests passed 7/7 in 87.430 s; the Make target
passed 7/7 in 90.648 s; the CLI passed `12 bounded GLES current-vertex-attribute
forms; source-only`. Full `make test` passed at the tested code revision,
including 1,127 Rust tests (3 ignored) and 338 web tests. Source limits passed
6/6; diff and roadmap checks passed.

Negative checks reject changed, reordered, missing, or cross-family templates;
heading, page, grammar-order, expansion-count, normalizer, family, route, or
profile drift; relocated authority or cache locators; semantic promotion;
duplicate, nonfinite, stale, or symlinked artifacts; unavailable ESSL/registry
substitutes; private-loader/cache substitution; ambient module aliases; and the
PATH-only extractor substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only expanded spelling, formal template,
physical page, section, and source order. It creates no attribute value,
conversion, shader input, query result, vertex processing, transform feedback,
guest/browser execution, support, conformance, certification, or performance
claim. It has no runtime Matrix or CTS execution.

Commit/push verification: `53ab6e42` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit 53ab6e42 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.6.2.
