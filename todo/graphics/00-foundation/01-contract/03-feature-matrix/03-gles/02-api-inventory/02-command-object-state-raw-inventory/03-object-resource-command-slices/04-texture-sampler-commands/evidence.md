# F03.3.2.2.3.4 evidence

Revision: ab12d9b62b507ced372c1a05f4dffc7ddd534bf3
Validation: four child receipts plus final local repository gates; no remote CI run was listed
Result: PASS
Artifacts: four self-hashed child raw inventories, 45 entries in total
Profile: GLES 3.2 formal texture/sampler declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.4, 2026-09-21.

Aggregate tested revision: `ab12d9b62b507ced372c1a05f4dffc7ddd534bf3`.
The four verified child code revisions are `519de165`, `71cdaf87`, `6bcb92fa`,
and `ab12d9b6`; each receipt records its own patch hash and source artifact hash.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The aggregate is four disjoint raw-only inventories: 14 texture/sampler object
and parameter forms, 7 image/copy/subimage literals, 12 compressed/storage/
buffer/mipmap/image literals, and 12 texture parameter/query expansions. Every
child binds the same authority/cache/domain/grammar/ledger identities and keeps
only expanded spelling, formal declaration, physical page/span, section, source
order, and when applicable source family.

Child receipts and artifacts:

- [objects and sampler parameters](01-texture-sampler-objects-parameters/evidence.md) — 14 entries
- [image, copy, and subimage](02-texture-image-copy-subimage-commands/evidence.md) — 7 entries
- [extended texture commands](03-compressed-storage-buffer-mipmap-image-commands/evidence.md) — 12 entries
- [texture parameter/query templates](04-texture-parameter-query-templates/evidence.md) — 12 entries

Reproduce from repository root:

```sh
make graphics-gles-texture-sampler-object-parameter-test
make graphics-gles-texture-image-copy-subimage-test
make graphics-gles-texture-extended-command-test
make graphics-gles-texture-parameter-query-template-test
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: all four focused child tests passed 6/6; their direct CLIs
passed 14, 7, 12, and 12 bounded raw forms respectively. The final full
`make test` passed, including 1,127 Rust unit tests (3 ignored) and 338 web
tests. Source limits passed 6/6; diff and roadmap checks passed after receipt
completion.

Negative/reference checks in the child receipts reject source-window, gap,
heading, grammar, normalizer, route, family, and order drift; cross-child
intrusion; semantic promotion; unavailable substitutes; stale/duplicate/
nonfinite/symlinked artifacts; and cache/private-loader substitution.

Decision and limits: this aggregation does not add texture format, image,
pixel-layout, sampling, parameter/state, query-result, buffer, storage, mipmap,
image-access, synchronization, guest/browser execution, support, conformance,
certification, or performance claims. It has no runtime Matrix or CTS execution.

Commit/push verification: every listed feature revision was pushed to
`origin/codex/graphics-f01-baseline` with a matching remote SHA; each queried
`gh run list --commit` had no run. This aggregate receipt is committed separately.

Next ready task: F03.3.2.2.3.5.
