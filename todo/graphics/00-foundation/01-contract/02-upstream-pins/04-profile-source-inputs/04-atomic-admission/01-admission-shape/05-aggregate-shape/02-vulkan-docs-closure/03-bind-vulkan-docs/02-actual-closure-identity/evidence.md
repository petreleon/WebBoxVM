# F02.4.4.1.5.2.3.2 receipt — actual Docs identity grammar

Revision: `af343ce3024a778f9d8ef89616d93699619981d8` verified implementation
Validation: 29 focused Docs tests, 48 predecessor-contract tests, source limits, roadmap checker, and `make test`
Result: PASS
Artifacts: tracked witness header only; full source and two rendered trees remain ignored under `.artifacts/`
Profile: unadmitted successor grammar only; no F02/V1 cutover, guest API, browser, CTS, conformance, or performance claim

The grammar has disjoint `raw-source-input`, `derived-source-input`, and `rendered-output` records. Raw and derived
inputs retain F02.2's 8 MiB limit. The observed 10,377,052-B HTML is an output only, with a 16 MiB per-artifact bound;
the two-run tree has separate 32 MiB and 4,096-file bounds in an isolated successor cache namespace.

The witness pins root `vkspec.adoc` at `f84d432d5b8912362f96f581f29bbc4f3c8c7843`, its raw URL, SHA-256, byte
count, SPDX `CC-BY-4.0`, the official image/toolchain, deterministic container context, core recipe, and two equal
tree identities. It also retains the V1 rejected-predecessor adapter. Every public result has `admitted=False` and
`cutover_ready=False`; nothing can enter the active F02/V1 consumer path.

Focused tests: 8 positive, 12 hostile, and 9 identity-binding cases passed; the witness CLI passed. They cover duplicate
safe identities, output/source separation, immutable URLs and commits, root/predecessor substitution, schema aliases,
recipe/configuration/generation binding, explicit WSI/video/extensions boundaries, structural partial closures, and
active-looking states.

Validation commands (the Python tests ran from their named task directories) were:

```text
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_identity_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_identity_hostile_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_identity_binding_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_identity_contract.py vulkan_docs_build_witness.json
PYTHONDONTWRITEBYTECODE=1 python3 source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 python3 candidate_audit_test.py
PYTHONDONTWRITEBYTECODE=1 python3 spec_include_test.py
PYTHONDONTWRITEBYTECODE=1 python3 boundary_test.py
PYTHONDONTWRITEBYTECODE=1 python3 post_cutover_test.py
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
make test
```

Predecessor tests passed 15+10+6+9+8 cases; source limits passed 6/6; focused tests had 0 failures/skips; `make test`
exited 0 (its emulator-unit portion: 1,127 passed, 3 ignored). No Rust/Wasm or browser code changed, so `make web-pkg`
was not applicable.

Boundary: this is a grammar and actual build witness, not a claimed complete Docs closure. In particular, no full
resolved conditional or promotion-metadata selector manifest is tracked here; `.3` must capture and bind that scope,
`.4` must stage/re-hash its complete output-tree manifest, and `.5` must prove the resulting closure. Remote CI and
guest/browser/CTS/conformance/performance tests were not run.
