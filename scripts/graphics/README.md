# Local graphics check runner

`run.py` executes only checks explicitly selected from a caller-provided JSON catalog.
It has no built-in API profile, GPU capability, source-admission, conformance, or
performance inventory.

```sh
python3 scripts/graphics/run.py --root . --catalog checks.json --select local-unit \
  --result .artifacts/graphics/local-unit.json
```

The catalog uses schema 1. Each check has exactly `name`, `command`,
`expected_count`, `artifacts`, `tools`, and `prerequisites`. Commands and tools are
argument arrays and never pass through a shell. `artifacts` are relative paths whose
bytes are SHA-256 hashed after the child exits.

```json
{
  "schema": 1,
  "checks": [{
    "name": "local-unit",
    "command": ["python3", "-c", "print('WEBBOXVM_GRAPHICS_OBSERVED_COUNT=1')"],
    "expected_count": 1,
    "artifacts": [],
    "tools": [["python3", "--version"]],
    "prerequisites": [{"kind": "executable", "value": "python3"}]
  }]
}
```

The child must emit exactly one `WEBBOXVM_GRAPHICS_OBSERVED_COUNT=N` line across
standard output and standard error. A missing, zero, duplicate, or mismatched count
is `FAIL`, even when the child exits zero. The JSON receipt records the command,
revision, dirty-diff SHA-256, declared tool output, duration, exit status, counts,
output digests, and artifact hashes.

Prerequisites are `executable`, `asset`, `browser`, `hardware`, and `permission`.
Hardware names an opt-in environment variable; permission names a relative probe path
and access (`read`, `write`, or `execute`). Missing prerequisites yield `BLOCKED` and
exit 3, never `PASS`. Invalid catalogs or selections exit 2. A single failing child
preserves its exit status and standard streams; other runner failures exit 1.

## Automated source inventories

The five vertex/transform-feedback slices and four state/execution routes use a
shared admission/row-validation engine and finite batch manifests. Catalogs retain
their exact section boundaries, declarations and hostile tests.

```sh
make graphics-gles-vertex-inventory-check GRAPHICS_GLES_CACHE=/path/to/verified/cache
make graphics-gles-vertex-inventory-regenerate GRAPHICS_GLES_CACHE=/path/to/verified/cache
make graphics-gles-state-inventory-check GRAPHICS_GLES_CACHE=/path/to/verified/cache
make graphics-gles-state-inventory-regenerate GRAPHICS_GLES_CACHE=/path/to/verified/cache
```

Vertex regeneration writes JSON into `.artifacts/graphics/vertex-inventory-regenerated`;
it leaves the checked-in artifacts and roadmap status untouched. Every slice must
pass before export starts. The batch report includes each inventory hash and count.
Domain/grammar/ledger proofs are reused only inside that invocation: admitted PDF
bytes and authority are rechecked for each slice, and all source inputs are hashed
before and after the batch. Changed inputs abort publication. No cached success
survives into another invocation.

Use `python3 scripts/graphics_inventory_batch.py --help` for task selection
and a custom output directory. The default group remains `vertex`; `--group state`
selects exactly the four existing state/execution routes. A state batch reports
each slice's completeness and missing/incomplete tasks. The lifecycle slice has
12 explicit object/name/shared-context rules and still has pending state routes;
successful extraction does not complete that leaf or its parent.

The other state routes contain 55 draw/raster/compute declarations, six pixel
commands and 24 debug/special/context-query declarations. Readable, self-hashed
source-family fragments preserve every row and its global source order under a
small aggregate index. Empty fragments explicitly cover inspected source windows
with no formal declarations. Reset declarations already assigned to the generic
route retain their exact existing receipt and are rechecked against the same PDF.

Formal extraction uses physical-page ranges, section fences, sealed declaration
signatures and an independent command-name index crosscheck. Pseudo-commands are
excluded only with explicit normative reasons. PDF typographic spellings retain
their exact quoted declarations and visually confirmed C names.

`make graphics-inventory-automation-test` covers batch failure, changed inputs,
unsafe destinations, real PDF extraction and stale/missing/substituted family
fragments. These are source checks; they do not establish API behavior or graphics
performance.
