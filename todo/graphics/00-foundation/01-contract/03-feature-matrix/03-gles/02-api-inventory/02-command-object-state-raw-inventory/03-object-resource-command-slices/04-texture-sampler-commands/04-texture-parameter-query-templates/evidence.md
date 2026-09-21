# F03.3.2.2.3.4.4 evidence

Revision: ab12d9b62b507ced372c1a05f4dffc7ddd534bf3
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_texture_parameter_query_template_inventory.json`
(`22694607d52312a42bcb00445b9551c12f948fed1aba29f2392211f5e6eece69`)
Profile: GLES 3.2 raw texture parameter/query templates only; promotion disabled

Task ID and date: F03.3.2.2.3.4.4, 2026-09-21.

Tested code commit: `ab12d9b62b507ced372c1a05f4dffc7ddd534bf3`, parent
`408dc27eb62ee75928b4535cedc848f89f71874a`; patch SHA-256:
`2ba800aefecee7cbf051a32fa4f15a0e648d9dfb3133a1fcca94d28a25c687df`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, authority hash
`0e7ee3270ca2c527ffafb366f216839a8d180dfc3777d2ef2b2a2a532023c7ae`.
The retained cache boundary is
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`.
Guest image/build, browser/adapter/driver: not applicable to this source-only task.

The raw set expands six formal templates into 12 ordered C names:
`TexParameteri`, `TexParameterf`, `TexParameteriv`, `TexParameterfv`,
`TexParameterIiv`, `TexParameterIuiv`, `GetTexParameteriv`,
`GetTexParameterfv`, `GetTexParameterIiv`, `GetTexParameterIuiv`,
`GetTexLevelParameteriv`, and `GetTexLevelParameterfv`. Its raw-entry hash is
`e7ecebc2a881e4c513e620cb65cfe293da17eca29d04d0c6488d3ff45ad65339`.
The sealed grammar window is source orders 17–22, on pp.206, 209, and 210,
under sections 8.10, 8.11.2, and 8.11.3. The integer parameter template keeps
the source spelling `uint texture`; it is not rewritten as a sampler form.

Reproduce from repository root:

```sh
make graphics-gles-texture-parameter-query-template-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/04-texture-sampler-commands/04-texture-parameter-query-templates/gles_texture_parameter_query_template_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed
`12 bounded GLES texture parameter/query forms; source-only`. Full `make test`
passed, including 1,127 Rust unit tests (3 ignored) and 338 web tests. Source
limits passed 6/6; diff and roadmap checks passed.

Negative checks reject missing/reordered templates; source page, section, heading,
template, grammar-order, expansion, normalizer, domain, or route drift; sampler
intrusion; semantic promotion; duplicate, nonfinite, or symlinked artifacts;
unavailable ESSL/registry substitutes; and private-loader or cache substitution.
The artifact is self-hashed and raw-only.

Decision and limits: this records only formal template spelling, expansion,
declaration, physical page, section, and source order. It creates no parameter
value, query-result, type, texture-state, level, ESSL, guest/browser execution,
support, conformance, certification, or performance claim. It has no runtime
Matrix or CTS execution.

Commit/push verification: `ab12d9b6` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list
--commit ab12d9b6 --limit 10` returned no runs.

Next ready task: aggregate F03.3.2.2.3.4 receipt.
