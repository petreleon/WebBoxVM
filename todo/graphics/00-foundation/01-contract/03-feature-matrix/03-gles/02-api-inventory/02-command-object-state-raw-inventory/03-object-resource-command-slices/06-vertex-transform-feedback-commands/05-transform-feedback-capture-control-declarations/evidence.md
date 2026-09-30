# F03.3.2.2.3.6.5 evidence

Revision: `96f2d56a7924e723af2e76c73dff60719225af6b`
Validation: focused source-inventory suite and CLI; aggregate integration belongs to the parent task
Result: PASS
Artifacts: `gles_transform_feedback_capture_control_raw_inventory.json`
(`cdfede659d7845a8cc6d7c8806b8a081318ac9a8d956bce4e5fb6a16acf3af9f`)
Profile: GLES 3.2 raw capture-control declarations; promotion disabled

Task ID and date: F03.3.2.2.3.6.5, 2026-09-30.
Baseline was clean; initial dirty-diff SHA-256:
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Before implementation, planned focused command from repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/05-transform-feedback-capture-control-declarations/gles_transform_feedback_capture_control_inventory_test.py
```

Expected: at least six nonzero positive/boundary/negative tests and exactly four
literal declarations, in source order, on physical p.357 / section 12.2.2. Tests
must reject missing/reordered declarations, incorrect signatures, declarations
outside the heading/page boundary, neighboring object commands and semantic claims.

Actual: nine tests passed, zero failed/skipped, exit 0 in 4.864 seconds. The
focused suite first revalidates the real pinned source, authority, domain map,
grammar, ledger and raw artifact through the shared `InventoryEngine`, then
reuses that verified source text for independent leaf-specific negative checks.
Catalog extraction uses the fixed private `scripts/graphics/inventories/pdf.py`
helper; the authority/cache/row/JSON machinery is shared rather than copied here.

The inventory has four ordered entries and `raw_only: true`,
`promotion_allowed: false`; its inner inventory SHA-256 is
`cff1d68a183f61c4f16208a55e2f6acd5171814e471078318ccdfde56ebdde81`.
The raw-entry SHA-256 is
`7e07e7e800b5d9256ef10c3066dc4f56f0fe50c0b2c8d06f0a02ed5b449be788`.

Tested working-tree code manifest SHA-256:
`1c6ae895418dffa429018d22a514baf6316c0fcd0978a4cb3091268b82047a66`.
It hashes sorted JSON of repository-relative paths to file SHA-256 values for
the three leaf Python files and the eight shared files `__init__.py`, `engine.py`,
`json_output.py`, `pdf.py`, `source.py`, `snapshot.py`, `vertex_batch.py`, `bootstrap.py`,
and the fixed `scripts/graphics_inventory_batch.py` entrypoint, using JSON sort_keys
and compact separators.
The initial clean baseline was `140c389cbd23d32eed4c2c75f780960aeae0e614`.

Tool versions: Python 3.14.6 and Poppler `pdftotext` 26.02.0. The CLI validation
command is:

```sh
python3 -B todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/06-vertex-transform-feedback-commands/05-transform-feedback-capture-control-declarations/gles_transform_feedback_capture_control_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
```

Expected/actual CLI: `PASS: 4 bounded GLES forms; source-only`, exit 0.
Integration target handed to the parent: `graphics-gles-transform-feedback-capture-control-test`.

Source cache was absent and restored from the immutable official Khronos URL:
`https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/specs/es/3.2/es_spec_3.2.pdf`.
Exact 2,198,754 bytes and SHA-256
`5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`
were checked before writing to the external identity-derived cache. The existing
cache verifier accepted 601 physical pages and the locator
`gles32-pdf-v1:page=357;section=12.2.2`.

Reference inspection: Poppler raw extraction and visual rendering of physical
p.357 (printed p.339) agree on the four exact declarations beneath the heading
`12.2.2 Transform Feedback Primitive Capture`. All return `void`; only
`BeginTransformFeedback` takes `enum primitiveMode`. The neighboring physical
p.358 contains continuation and primitive/capture semantics, which remain outside
this declaration slice.

Negative checks mutate the actual source excerpt, rather than trusting a test
name: missing/duplicate/reordered declarations; altered return/parameter forms;
missing heading/footer; declarations moved outside the section; added object,
vertex-array, varying, draw or extension forms; widened page scope; rehashed
cross-family declarations; wrong C prefix; and changed family order/route.
The source footer closes this physical-page slice without importing p.358 state.
Rehashed `capture_state` and `profile_support` promotions fail against the already
validated source snapshot without repeating upstream PDF work. This leaf records
declarations only. No cache or test prerequisite was skipped. Leaf diff whitespace,
Python syntax and the 180-physical-line limit all passed, exit 0.

Guest image/build, browser/adapter/driver, runtime execution, software fallback
and performance protocol: not applicable to this source-only task.
No object behavior, captured output, support, conformance, certification, runtime
Matrix or CTS claim is made. Commit/push and final integration are owned by the
parent task and are not yet verified.

Reproduction: restore the exact pinned PDF to
`/private/tmp/webboxvm-f0341.cqT6ZX/webboxvm-graphics/f02/gles-32-spec/5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c.source`
and run the commands above. `WEBBOXVM_GRAPHICS_CACHE_ROOT` can select another
verified external cache for the focused test; no in-repository PDF is accepted.
Large rendered source captures are disposable; the pinned source and commands
recreate them. No remote CI run or publication is claimed by this receipt.
