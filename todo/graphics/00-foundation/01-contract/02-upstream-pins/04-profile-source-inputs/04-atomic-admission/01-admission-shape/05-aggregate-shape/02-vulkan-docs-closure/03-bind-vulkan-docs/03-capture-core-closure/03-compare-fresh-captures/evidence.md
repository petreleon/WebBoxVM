# F02.4.4.1.5.2.3.3.3 receipt — independently recorded Docs captures

Revision: `a71f33b5` tested comparison revision (implementation `e75fa430`)
Validation: 16 focused comparison tests, live runner and contract receipt, source limits, roadmap checker,
`git diff --check`, and `make test`
Result: PASS
Artifacts: [compact tracked receipt](vulkan_docs_core_input_comparison.json); the two captures and regenerated
ignored receipt remain under `.artifacts/graphics/f02.4.4.1.5.2.3.3.1/` and
`.artifacts/graphics/f02.4.4.1.5.2.3.3.3/comparison/receipt.json`
Profile: Vulkan 1.4 core input comparison only; unadmitted and not cutover-ready

## Recorded comparison

The public route binds `observer-a` and `observer-b` through the pinned `.3.2` input/scope binder before it examines
locations or creates a receipt. It accepts two separately stored, canonical, non-symlink recorded capture locations:
`sources/observer-a` with `runs/observer-a-r5`, and `sources/observer-b` with `runs/observer-b-r1`. It requires
regular `vkspec.adoc` and `generated/out/html/vkspec.html` members, distinct source/output roots, and distinct
`run.json`, normalized-input, I/O-trace, and include-trace files. It does not run Git, traverse a source tree, read
source/output payloads, count a generated tree, or hash a generated tree in this comparison.

The two selected headers have distinct run and I/O identities but equal normalized input
`ec26645c8f8b4560f2882240ae4fab0ffb51ca4bacdc04f6d9ae363f48affc52`, include trace
`9bdd4e1a79ffc224832ec58763034dcf87e9660af7652fda5193891c226ec024`, input-manifest
`1896b1a211ecf82ff8b7826718ed797083fa80f41eba0a985376cef124b3056c`, and semantic scope
`9c0e5fb53986b22ee5d774de350127cdecfe11c021d39106a20a106203ef3339`. The tracked comparison identity is
`f76e489565365d2b326185d2f4d511422c8b83b6b084fee0f90cd3b5646cc9ce`; the regenerated ignored receipt SHA-256 is
`90862e9ebbc3c7f93bc4bcf9880b43bbc86741802c3f24d260ab7781b52ab748`.

The compact receipt retains nonzero evidence: 2 captures, 298 raw inputs, 1,462 derived inputs, 1,973 includes,
7 conditions, 4 promotion selectors, 9 extension controls, and 42 images. It rejects a mismatch in normalized,
input-manifest, raw, derived, include, producer, configuration, condition, scope, build-witness, include-trace, or
observation identity, as well as reused provenance, required-artifact hard links, nested/root/source/output symlinks,
missing primary members, stale self-seals, and an active-looking receipt. The public-wiring tests also prove that
both pinned bindings occur before the location gate and that a rejected first binding cannot reach it.

## Output boundary and limits

The relation retains the known `2,530 / 17,019,466 B /
26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32` output witness and primary HTML
`896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041` only by matching prevalidated capture
metadata to the reviewed constants. The count and byte total are copied witness facts; this child neither verifies
payload content again nor stages it. Child `.4` owns that work.

These are independently stored, non-aliased recorded observations, not a new replay. Freshness, clean checkout state,
and execution provenance remain evidence from `.3.1`; this comparison cannot prove time separation, different hosts,
or the original filesystem provenance of a byte-identical copy at a different path. Its structural checks establish
only retained regular primary members and non-aliasing. It changes no inventory, candidate, F03, guest, browser, CTS,
conformance, or performance state.

## Validation

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest vulkan_docs_compare_test.py vulkan_docs_compare_hostile_test.py vulkan_docs_compare_location_test.py -v
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_compare_run.py --observation ../01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json --artifact-root <canonical-ignored-artifact-root> --output <ignored-receipt>
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_compare_contract.py vulkan_docs_core_input_comparison.json ../01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json <canonical-ignored-artifact-root>
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
make test
```

The focused run passed 16/16 test methods: five receipt/public-route tests, five semantic hostile tests, and six
location/alias tests. The live runner and tracked receipt contract both reported two captures and zero cutover-ready.
Source limits passed 6/6. `make test` passed 1,127 emulator tests (3 ignored), all boundary/roadmap checks, and 337
Node tests. No Rust/Wasm/browser files changed, so `make web-pkg` was not applicable. Remote CI was not run.

Next ready task: `F02.4.4.1.5.2.3.4` — stage and verify build witnesses.
