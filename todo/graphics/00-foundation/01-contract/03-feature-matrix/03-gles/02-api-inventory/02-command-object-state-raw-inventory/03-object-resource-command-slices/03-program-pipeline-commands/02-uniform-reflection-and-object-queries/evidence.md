# F03.3.2.2.3.3.2 evidence

Revision: f1ec38a1bd33714243d45db36479e116c4a3dae9
Validation: local focused and repository gates; no remote CI run was listed
Result: PASS
Artifacts: `gles_uniform_query_raw_inventory.json`
(`60f0051d56151b3c0f72179d0534d23c99a5e07dd27d08cd571b1ba3d05c24bf`)
Profile: GLES 3.2 raw declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.3.2, 2026-09-21.

Tested code commit: `f1ec38a1bd33714243d45db36479e116c4a3dae9`, parent
`c23ef0cb775f48c5ce7ccf3ab6bcd386ae4ff2f4`; patch SHA-256:
`21171f0d6832c4806b7b3723a97c8d98386bab5f08db63fe2a3f6c558e68199f`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The raw set has 23 literal declarations in PDF source order across pp. 123–125,
133, and 145–152, sections 7.6, 7.6.3, and 7.12. Its raw-entry hash is
`cc8775b982ebba37e91068f1407242163e0d3005c5f071e71135a0b4e1980ab5`.
The validator reads only that sealed noncontiguous page set, preserves p.125
`sizei length`, and fences p.133/p.145 declarations after their section markers.

Reproduce from repository root:

```sh
make graphics-gles-uniform-query-inventory-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/03-program-pipeline-commands/02-uniform-reflection-and-object-queries/gles_uniform_query_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 6/6; the CLI passed `23 bounded GLES
uniform/query literals; source-only`. Full `make test` passed, including 1,127
Rust unit tests and 338 web tests. Source limits passed 6/6; diff and roadmap
checks passed.

Negative checks reject altered, omitted, reordered, cross-family, or rerouted
literals even when resealed; a stale page policy; p.133 section relabeling;
`MemoryBarrier`; a fabricated query-result field; stale, duplicate, nonfinite,
or symlinked artifacts; stale domain order/route; unavailable ESSL substitutes;
and cache/private-loader substitution. The artifact is self-hashed and raw-only.

Decision and limits: this records only declaration spellings and source locations.
It creates no value, precision, state, memory-barrier, lifecycle, ESSL, shader,
program, pipeline, guest/browser, support, conformance, certification, or
performance claim. It has no Matrix or CTS execution.

Commit/push verification: `f1ec38a1` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
f1ec38a1 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.3.3.
