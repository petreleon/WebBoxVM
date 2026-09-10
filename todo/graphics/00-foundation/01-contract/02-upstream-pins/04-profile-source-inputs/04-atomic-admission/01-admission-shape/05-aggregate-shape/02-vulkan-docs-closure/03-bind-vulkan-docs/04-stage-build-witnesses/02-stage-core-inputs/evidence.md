# F02.4.4.1.5.2.3.4.2 receipt — staged Vulkan Docs core inputs

Revision: `9f5e941ff40ed051dac0b788c169791ca63984fd` verified implementation
Validation: 14 focused tests, 56 prerequisite tests, live stage/verify CLI, source limits,
roadmap checker, `git diff --check`, and `make test`
Result: PASS
Artifacts: temporary private cache only; it was deleted after exact-count verification
Profile: Vulkan 1.4 core recorded-capture input staging only; unadmitted and not cutover-ready

## Result

The public one-flow contract descriptor-read both selected providers for each of the 1,760
bound members: `298` raw plus `1,462` derived records, or exactly `3,520` physical reads.
It writes one canonical copy under separate `inputs/raw/` and `inputs/derived/` namespaces,
then rehashes every payload, confirms the exact input-subtree inventory, and publishes only a
self-hashed `staged-inputs` receipt. The live run produced `1,761` files: the 1,760 payloads
plus that receipt; it produced no `outputs/` payload and no `.complete.json` marker.

The canonical input manifest remains
`1896b1a211ecf82ff8b7826718ed797083fa80f41eba0a985376cef124b3056c`:

- raw: 298 members, 15,779,568 bytes;
- derived: 1,462 members, 2,816,281 bytes;
- canonical total: 1,760 members, 18,595,849 bytes; paired provider reads: 37,191,698 bytes.

The temporary live receipt had SHA-256
`caf23fb89062418579e4cc4aa071b0133a1215088a01347436734f0f070de72a` and plan digest
`5dcc55e3cd979010d73142e7dde13361e0b36f64ff42aaa1e436a8a5eab5de77`.

## Contract and negative coverage

Provider roots, cache root, and every traversed directory are no-follow descriptor paths owned
by the effective user and not group/world writable. Provider reads require regular files,
`nlink == 1`, fixed size, ctime/inode stability, and the bound SHA-256. Both recorded captures
must return identical bytes before atomic no-overwrite publication. The cache root and every
payload parent stay descriptor-anchored across the transaction; named-root and nested-parent
swaps fail. The stage refuses any prior worktree, payload, receipt, or completion-looking marker.

The receipt recomputes the exact live member tuple and canonical JSON, so selector changes,
duplicate JSON keys, integer/boolean aliases, count changes, scope/comparison changes, or a
resealed receipt are rejected. The input inventory is bounded and exact, rejecting unlisted,
nonregular, hard-linked, over-deep, or excess entries while intentionally ignoring sibling
`outputs/` and `markers/` namespaces reserved for later children. A repeated derived digest has
36 selectors; cache locations remain injective by `(kind, selector)`, never digest alone.

Focused hostile cases cover B-only divergence, unsafe provider files and roots, same-size rewrite,
provider/cache root and nested-parent swaps, raw/generated cross-routing, `generated/out`, caps,
untrusted plans, existing partial worktrees, stale receipts, cache mutation, and unlisted inputs.

## Validation

Focused command from `02-stage-core-inputs`:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  vulkan_docs_input_cache_test.py vulkan_docs_input_hostile_test.py \
  test_vulkan_docs_input_contract.py -v
```

It passed 14/14. The public integration test instruments exactly 3,520 provider reads, proves
36 shared-digest selectors retain 36 targets, refuses a second stage, and detects a later cache
mutation and an extra input member.

Prerequisites passed: staging contract 15/15, actual identity 30/30, bound input scope 10/10,
and independent comparison 16/16. `cargo test -p emulator --test source_file_limits --quiet`
passed 6/6. `make test` exited 0: 1,127 emulator unit tests passed (3 ignored), boundary and
source-limit checks passed, the graphics roadmap checker passed, and Node passed 337/337.
No Rust, Wasm, or browser source changed, so `make web-pkg` was not applicable.

The final live commands used an empty `mktemp -d /private/tmp/webboxvm-input-stage-final.XXXXXX`
root and printed both times:

```text
INPUT-STAGE: staging-only-unadmitted, 298 raw, 1462 derived, 0 markers
```

The root was then removed. First failing subcheck: none. Remote CI was not run.

## Boundary and next work

This is a recorded-capture snapshot, not a fresh official build, output witness, reusable marker,
or graphics compatibility/performance result. A same-UID mutation after the final check cannot be
retroactively prevented; every verification rehashes and inventories the named input snapshot.
A byte-identical same-UID artifact-root replacement is content-equivalent but cannot prove a
different physical origin; divergent or substituted content is rejected before staging.

Implementation commit: `9f5e941ff40ed051dac0b788c169791ca63984fd`; this receipt/status update
is committed separately. Next ready task: `F02.4.4.1.5.2.3.4.3` — stage both bounded output
witnesses. The parent remains open until its remaining children complete.
