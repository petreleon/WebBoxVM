# F02.4.4.1.5.2.3.3.2 receipt — bound core Docs input scope

Revision: `0c0302f5` bound-byte hardening after implementation `3006a37f`
Validation: 4 positive and 6 hostile scope tests, selected-artifact contract CLI, source limits, roadmap checker,
`git diff --check`, and `make test`
Result: PASS
Artifacts: compact tracked receipt only; the full bound manifest remains ignored at
`.artifacts/graphics/f02.4.4.1.5.2.3.3.2/observer-a/scope-manifest.json`
Profile: one capture's input/scope binding only; unadmitted and not cutover-ready

## Bound capture

[`vulkan_docs_core_input_scope.json`](vulkan_docs_core_input_scope.json) binds only `observer-a` from the prior
tracked observation: run `4fdd14f649aaa2e346bd5e0d1d2f77248f4c5b0dab1ce596101872520f7f48f8`, normalized artifact
`ec26645c8f8b4560f2882240ae4fab0ffb51ca4bacdc04f6d9ae363f48affc52`, I/O trace
`f263e0aa51baaafce451dd27ebad3f9ab387cdfa5aa14264090744f02342e13a`, include trace
`9bdd4e1a79ffc224832ec58763034dcf87e9660af7652fda5193891c226ec024`, observation
`dbd75e819e3021e28e2bdb85eeccd675a688ffdb6692fddcd8ff58a221363554`, and input manifest
`1896b1a211ecf82ff8b7826718ed797083fa80f41eba0a985376cef124b3056c`.

The public binder pins that observation digest, both reviewed run identities, counts, phase map, input-manifest, and
include identity before it accepts the ignored artifact. It resolves only regular, non-symlink artifact files; it
parses the normalized JSON once within 1 MiB and seals the exact same bytes, while streaming each required trace under
its 64 MiB bound. A syntactically valid self-sealed replacement header or added generated extension branch therefore
cannot replace this capture identity.

The compact receipt has 298 raw and 1,462 derived identities, 1,973 resolved includes, and semantic scope identity
`9c0e5fb53986b22ee5d774de350127cdecfe11c021d39106a20a106203ef3339`. Raw entries derive a pinned
`raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/f84d432d5b8912362f96f581f29bbc4f3c8c7843/...` identity;
derived entries bind generation `official-vulkan-docs-core-html-1-4-362`. Every input is a safe selector with a
nonzero SHA-256 and the F02 8 MiB cap. Rendered `out/` members cannot enter either input namespace.

The producer identity retains source tree
`99cfd3132413891764f98f246567d0ce0344f2f0bb5499bfbc7fc47646fa122b`, exact producer argv
`9a985b9e36cdf53e7baa0a3b30c9df41d33d8feb04f8f08904532a807b4dbf3c`, all six nonzero phases, and build
configuration `733f0e65d740e6dded26fc0c525987fa81a6e6c51bf2f757995f7923586b472e`. The known build witness is bound as
`8cfab6fe4527f3973d7d7923ffe0922ab689fc2daf2f6e955ff7603fc8ba78d6`, not treated as an input payload.

## Scope treatment

The binder requires both raw extension-control sources and seven generated extension-control metadata inputs, plus
the four `promoted_extensions_VK_VERSION_1_[1-4].adoc` inputs and `generated/specattribs.adoc`; each must have an
Asciidoctor read and a resolved include. These controls are retained as core build controls, not mistaken for active
individual-extension semantics.

It records 42 raw SVG image inputs (1,182,290 B) from I/O evidence. It requires zero records and zero include events
for WSI, video, all `appendices/VK_*`/`chapters/VK_*` branches, and 19 exact generic conditional-extension targets
such as `chapters/descriptorheaps.adoc` and `chapters/raytracing.adoc`. It deliberately does not deny
`chapters/private_data.adoc` or `appendices/memorymodel.adoc`: their core-version alternatives are active.

As a defense independent of the pinned normalized identity, the semantic guard also rejects generated extension
interfaces/metadata and vendor-suffixed generated API or validity members. The hostile `...PropertiesNV.adoc` case
refreshes its local fixture capture and still fails specifically as individual-extension semantics.

## Validation

```text
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_scope_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_scope_hostile_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_scope_run.py --observation ../01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json --artifact-root <ignored-artifact-root> --run-id observer-a --output <ignored-manifest> --receipt-output <ignored-receipt>
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_scope_contract.py vulkan_docs_core_input_scope.json ../01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json <ignored-artifact-root> observer-a
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
make test
```

The positive suite passed 4/4 and the hostile suite passed 6/6. The hostile cases cover partial/root-only scope,
missing controls/promotions/images, duplicate or unordered records, traversal/URL-reserved selectors, over-cap input,
output injection, stale producer/receipt configuration, WSI/video, generic/prefix/generated inactive extension
branches, and a self-sealed observation header. Source limits passed 6/6. No Rust/Wasm or browser code changed, so
`make web-pkg` was not applicable.

A temporary copy of the selected artifact was additionally tampered in three ways: changing only
`ignored_runtime_reads`, changing one I/O-trace byte, and removing the include trace. The selected-capture binder
rejected all three before returning an unadmitted scope.

## Boundary

This leaf does not compare `observer-a` with `observer-b`, re-hash or stage the rendered tree, prove complete closure,
admit a source, or change any F02/V1 inventory, guest, browser, CTS, conformance, or performance state. The next child
`.3.3` owns the independent-capture comparison; output-tree payload staging remains for child `.4`.
