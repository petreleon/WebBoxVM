# F03.3.2.2.4.4 evidence

Revision: `920328321f5c3fafee1bade3d823b1c26458abcb` plus the recorded working-tree source manifest
Validation: focused positive/negative source checks and finite state batch
Result: PASS focused; final repository integration and commit/push remain with the parent
Artifacts: `gles_debug_special_query_command_raw_inventory.json` and its source-family fragments
Profile: GLES 3.2 raw formal declarations; promotion disabled

Task/date: F03.3.2.2.4.4, 2026-09-30. The shared workspace was already dirty;
no unrelated runtime/browser changes were removed. Observed dirty-diff SHA-256:
`dbc7f1457a3f2b6ad5634ecdadb546304fd99aa393dacda4c5b51bcc3cc19c5d`.

Focused protocol from the repository root:

```sh
python3 scripts/test_graphics_state_declarations.py DebugTests
```

Expected/actual: 7 nonzero tests, 24 raw C declaration forms, zero failed/skipped.
The measured combined command below passed43 tests in 17.154 seconds, exit 0:

```sh
python3 -m unittest scripts.test_graphics_inventory_pdf scripts.test_graphics_inventory_storage scripts.test_graphics_inventory_batch scripts.test_graphics_state_declarations
```

Counts: PDF 11, fragment storage 6, original batch 6 and Draw 7 / Pixel 6 / Debug 7.
These are source checks. Lifecycle's separate 12 checks are recorded in its receipt.
The tests validate the real exact PDF/cache/authority/domain/grammar/ledger and
stored artifacts, then mutate actual source text, windows, routing and the index.
They reject missing/duplicate/extension/changed-return declarations, shortened
windows, changed family order/scope/route, missing index witnesses and unadmitted
registry sources. Storage checks reject rehashed stale row/count/order/name and
promotion changes, missing/symlink fragments and redirected aggregate references.

The complete debug, special and context-queries routes span chapters 18–20. The reset declaration remains in its existing generic-context route and exact completed F03.3.2.2.3.1 inventory.

Ten debug declarations, Hint and 13 context queries are extracted. Physical p.458 visually confirms GetBooleani_v/GetIntegeri_v/GetInteger64i_v; their quoted PDF declarations preserve the extraction spaces. GetString/GetStringi retain the normative ubyte pointer return, and GetObjectPtrLabel retains the source size *length spelling. GetGraphicsResetStatus p.34 is rechecked against its exact previous inventory bytes/hash and the same normative PDF; it is not duplicated or rerouted.

Shared extraction scans physical-page ranges and exact heading/end fences,
retains every formal signature and global source order, and compares the result
with a separately sealed command-name PDF-index set. Every raw row binds its own
source family/order and physical page/section. Source-family fragments and their
small aggregate index are each self-hashed and compared with fresh extraction;
empty inspected families are retained. No parser or full validator is copied here.

Exact source: official immutable Khronos GLES 3.2 PDF, revision
`1cdd228e34966dd6b95bd203e9f84faba0f371a1`, 2,198,754 bytes, 601 physical pages,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`.
External cache: `/private/tmp/webboxvm-f0341.cqT6ZX`; override focused checks with
`WEBBOXVM_GRAPHICS_CACHE_ROOT` after restoring/verifying those exact bytes.
Toolchain: Python 3.14.6, Poppler 26.02.0; fixed private extractor with sterile PATH.

Stored aggregate-index SHA-256 (`inventory_sha256`):
`9bab0a601bcffaf498ba952df9df0f2239e6fde2c07aad88b3b81bfd663e9ab5`.
Reconstructed inventory SHA-256 (`aggregate_inventory_sha256`):
`cdab7a42c9aee8be8ef50bbf97e2708c774d35fc4e23ad4126866d28aa93e6ec`.
Raw row-array SHA-256:
`95b292a6fe62a99a7f7fd7d29788626bde335ce923dd82b8a9550a8472e7957e`.

Tested 21-file Python source manifest SHA-256:
`e768511625edd3c41fd6be8903a81dc1c69784416dd143af70bcae59ee44ddb9`.
It hashes sorted JSON of repo-relative paths to file SHA-256 values using compact
separators. The manifest is `.artifacts/graphics/2026-09-30-state-batch/code-manifest.json`;
it covers all shared inventory Python files, fixed entrypoint, three new wrappers/
catalogs and the three changed focused test files. It excludes unrelated GPU edits.

Finite batch commands, observations and durable reproduction are in the
[parent aggregate receipt](../evidence.md). Check and regeneration validated all
four state tasks / 97 facts; all 20 regenerated files match checked-in bytes. The
lifecycle slice and parent remain incomplete. No Matrix/CTS/owner/claim field,
state behavior, ESSL semantics, guest/browser execution, support, conformance,
certification or performance result is inferred. Final full integration and
commit/push are not claimed by this receipt.
