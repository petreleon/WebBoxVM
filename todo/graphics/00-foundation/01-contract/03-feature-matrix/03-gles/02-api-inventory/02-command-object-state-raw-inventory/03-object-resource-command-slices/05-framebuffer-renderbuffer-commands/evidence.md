# F03.3.2.2.3.5 evidence

Revision: 140a2b5482ed5ca50f207cb454c093a2a5d93fac
Validation: three child receipts plus final local repository gates; no remote CI run was listed
Result: PASS
Artifacts: three self-hashed child raw inventories, 19 entries in total
Profile: GLES 3.2 formal framebuffer/renderbuffer declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.5, 2026-09-21.

Aggregate tested revision: `140a2b5482ed5ca50f207cb454c093a2a5d93fac`.
The three verified child code revisions are `f9d53b7d`, `add97482`, and
`140a2b54`; each receipt records its own patch hash and source artifact hash.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The aggregate is three disjoint raw-only inventories: 7 framebuffer-object
forms, 7 renderbuffer-object forms, and 5 attachment/status forms. Every child
binds the same authority/cache/domain/grammar/ledger identities and keeps only
expanded spelling, formal declaration, physical page/span, section, source
order, and when applicable source family.

Child receipts and artifacts:

- [framebuffer objects](01-framebuffer-object-parameter-query-commands/evidence.md) — 7 entries, `0063c956…ffaa32`
- [renderbuffer objects](02-renderbuffer-object-storage-query-commands/evidence.md) — 7 entries, `2629280b…94ff3e`
- [attachments and status](03-framebuffer-attachment-status-commands/evidence.md) — 5 entries, `e2dc7ff9…7d63f8`

Reproduce from repository root:

```sh
make graphics-gles-framebuffer-object-parameter-query-test
make graphics-gles-renderbuffer-object-storage-query-test
make graphics-gles-framebuffer-attachment-status-test
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: all three focused child tests passed 7/7; their direct CLIs
passed 7, 7, and 5 bounded raw forms respectively. The final full `make test`
passed, including 1,127 Rust unit tests (3 ignored) and 338 web tests. Source
limits passed 6/6; diff and roadmap checks passed after receipt completion.

Negative/reference checks in the child receipts reject source-window, gap,
heading, witness, grammar, normalizer, profile, route, family, and order drift;
cross-child intrusion; semantic promotion; unavailable substitutes;
stale/duplicate/nonfinite/symlinked artifacts; and cache/private-loader
substitution.

Decision and limits: this aggregation does not add bind, storage, attachment,
image-selection, completeness, status-value, format, query-result, ESSL,
guest/browser execution, support, conformance, certification, or performance
claims. It has no runtime Matrix or CTS execution.

Commit/push verification: every listed feature revision was pushed to
`origin/codex/graphics-f01-baseline` with a matching remote SHA; each queried
`gh run list --commit` had no run. This aggregate receipt is committed separately.

Next ready task: F03.3.2.2.3.6.
