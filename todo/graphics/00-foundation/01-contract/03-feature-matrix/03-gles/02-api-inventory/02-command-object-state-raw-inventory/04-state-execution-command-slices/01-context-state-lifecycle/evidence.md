# F03.3.2.2.4.1 scoped evidence

Revision: `920328321f5c3fafee1bade3d823b1c26458abcb` plus the owned working-tree files
Validation: twelve focused source tests and exact-cache shared-engine CLI
Result: FAIL
Artifacts: self-hashed `gles_context_state_lifecycle_raw_inventory.json` plus eight source-family fragments
Profile: GLES 3.2 initial object-model/shared-lifecycle literal-trigger source rules; full leaf open

Task and date: F03.3.2.2.4.1, 2026-09-30. The working tree was already dirty with
other workers' source/runtime edits; no outside file was changed by this worker.
The observed tracked dirty-diff SHA-256 at verification time was
`0d5c12c55dd8fe7b3fb0cda2bff7cdc7cd88995c6f726d379d83e8d2beba6bb1`.
This is an observed verification-time hash, not a reconstructed initial baseline.

The [contract](contract.md) fixes a coherent initial extraction group rather than
claiming the whole lifecycle route. Expected/actual: twelve quotations, eight
domain bindings, five pending families, twelve nonzero tests, zero claim fields.
Twelve tests passed, zero failed/skipped, exit 0 in 5.895 seconds. The final focused
commands ran from `/Users/petreleon/code/WebBoxVM`:

```sh
python3 -B todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/04-state-execution-command-slices/01-context-state-lifecycle/gles_context_state_lifecycle_inventory_test.py
python3 -B todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/04-state-execution-command-slices/01-context-state-lifecycle/gles_context_state_lifecycle_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
```

CLI actual: `PASS: 12 bounded GLES forms; source-only`, exit 0. An earlier focused
run also passed twelve tests; the final run adds an actual section relocation and
state-table anchor substitution. Tests first validate the real pinned source and
artifacts, then reuse those source pages for adversarial checks. No engine validator
was copied. The catalog and wrapper use fixed private paths; shared admission,
grammar, PDF extraction, compact JSON and fragment sealing are reused.

The source is the pinned Khronos GLES 3.2 PDF at immutable revision
`1cdd228e34966dd6b95bd203e9f84faba0f371a1`, exact 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`.
The admitted authority/cache/domain/grammar/ledger identities are checked by the
shared source engine against that exact root. No desktop, earlier GLES, registry,
ESSL or extension source substitutes for it. Tool versions: Python 3.14.6 and
Poppler `pdftotext` 26.02.0; the full PDF has 601 physical pages.

Reviewed physical pages: 24–26, 42–47 and 62–67. Their complete compact text is
sealed, including excluded generic/undefined paragraphs. Positive rules occur on
43/44/63/65/66, sections 2.6.1.1, 2.6.1.2, 5.1.2, 5.1.3, 5.3, 5.3.1 and 5.3.3.
Poppler renderings of these five pages (printed 25/26/45/47/48) were visually checked
against raw extraction. The quotations preserve qualifications about unbound
containers, other contexts, name generation and wait/completion behavior; source
line-break hyphens remain in the stored quotation. This is not a C-signature
inventory: no formal prototype is fabricated from a prose command mention.

The twelve quotations comprise four object-taxonomy rules and eight shared-context
rules. The bindings also retain fundamentals and all five pending state families.
The `source_coverage` ledger is explicitly incomplete. Empty family fragments
preserve their route/binding; they never imply zero mandatory source work.

Negative checks reject missing/repeated/reordered quotations, missing or changed
section/page fences, a quotation moved beneath a different heading, command/EXT
substitution, rehashed unrelated family/section/trigger catalogs, wrong grammar C
prefix, different profile/primary locator, rerouted/omitted/reordered domain
families, changed table anchors/reasons, rehashed complete/support/Matrix claims,
and rehashed fragment omission/prefixed commands. Wildcards and generic rules do
not become guessed API facts; the undefined feedback-loop clauses are excluded.

The aggregate raw-row SHA-256 is
`d3d6d96c8d3cccd04a6bc24f6fb734698255896d4d3bd8bee2504ccc470e2ab5`.
Aggregate inventory SHA-256:
`78a1ae0d7e9ac05ca669dea6871bac19158d86de7c20fbd41d74ff6396868149`.
Index self-hash: `f674df5c68ddfce9eb0b7f5a3108fcc6d0f0a25a92e8e518a09b996ee070c7a4`.
Index file SHA-256: `0e378635f0941909f6eb8038d5fc18c22a55411b1a35e44d07da2b14c3665c2c`.
Each of the eight referenced fragments carries a verified self-hash and row hash.

[validation-manifest.json](validation-manifest.json) records the owned Python files,
fixed shared package and batch entrypoint file hashes. Its sorted compact JSON
SHA-256 is `440311f5e304f31d20291a34f3faacef8c71a6f42c4991c5e0eb6e01b0cb9688`.
Owned Python syntax, leaf diff whitespace and the 180-physical-line limit passed.
The longest owned source/test file has 159 lines; index 132; populated fragments
57/77; empty fragments 24. Required aggregate checks belong to the root worker.

First unmet acceptance: complete assigned-source coverage. Remaining mandatory
families cover 10.1/10.6–10.7, chapter 11, 12.1/12.3–12.6, chapter 14, and tables
21.1–21.39. Their state commands, query declarations and triggered table rules
have not been extracted. Thus the full task is FAIL/open even though the bounded
initial extractor's focused checks pass. There is no external blocker claimed.

Guest image/build, browser/adapter/driver, runtime execution, memory-ordering,
fallback and performance: not evaluated by this source-only group. No Matrix,
CTS, implementation owner, support, conformance or certification claim is made.
Integration, commit and push are owned by the root worker and unverified here.
Suggested focused Make target: `graphics-gles-context-state-lifecycle-test`.

Reproduce by restoring the immutable PDF from
`https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/specs/es/3.2/es_spec_3.2.pdf`
to the external identity path
`/private/tmp/webboxvm-f0341.cqT6ZX/webboxvm-graphics/f02/gles-32-spec/5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c.source`.
The exact source and `pdftoppm -f PAGE -l PAGE -singlefile -png` recreate the
disposable `/private/tmp/webboxvm-lifecycle-{43,44,63,65,66}.png` reference captures.
No remote CI run or publication is claimed by this receipt.
