# F02.3.3.4.4 evidence

Revision: `b29a48b` baseline; scoped WebGPU boundary probe
Validation: seven hermetic boundary tests, current-inventory blocker probe, source limits, `make test`, roadmap, diff
Result: BLOCKED
Artifacts: `webgpu_generator_boundary.py`, its hermetic test, and this local inventory-only receipt
Profile: immutable-input eligibility boundary only; no WebGPU API, guest, browser, code-generation, or performance claim

Task ID and date: F02.3.3.4.4, 2026-09-09.

At the original probe, the F02.1 inventory-lock SHA-256 was
`cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b`. Its `webgpu-spec` entry
is a `webgpu` source family but has role `host API semantic reference; no generated code`.
Its `wgsl-spec` entry has `wgsl` source family and role `WGSL emitter semantic reference; no
generated code`. Neither is accepted as a WebGPU generator input. No manifest entry has the exact
designation `future WebGPU generator input`.

The original offline checker read only the local TOML inventory. It rejected any `wgsl` source before
it could be a WebGPU generator input; its seven hermetic cases included a temporary grammar candidate
with a generator role and proved that it was still rejected by source family. A separate temporary
future WebGPU-designated entry makes the blocker assertion fail, so this receipt cannot silently
remain valid after the inventory gains an eligible input. No upstream payload, network request,
record sidecar, generated protocol, or runtime artifact is used.

## Grammar-lock renewal

F02.3.3.4.1.2 now supplies the actual `wgsl-grammar-syntax` entry under the separate
`wgsl-grammar` family. Its current inventory-lock SHA-256 is
`db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e`; the committed-entry
boundary test rejects it as WGSL-derived. The real probe remains three BLOCKED lines: reference-only
`webgpu-spec`, WGSL-derived `wgsl-spec`, and no explicit WebGPU generator input. This renewed
inventory fact does not turn the BLOCKED result into a WebGPU implementation or runtime claim.

Commands, working directory, and actual result:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/webgpu_generator_boundary_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/webgpu_generator_boundary.py --manifest todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/manifest.toml
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Working directory: `/Users/petreleon/code/WebBoxVM`. The hermetic suite passed 7/7 and the source
limit suite passed 6/6. The real probe printed three `BLOCKED:` lines: `webgpu-spec` is
reference-only, `wgsl-spec` is WGSL, and no explicit WebGPU generator input exists. The full local
suite, roadmap checker, and whitespace check exit successfully after this receipt is present.

Blocker: F02.1 does not yet identify a reviewed immutable WebGPU generator source distinct from
the reference specifications. This is an intentional BLOCKED result, not a passing WebGPU feature.
Keep this leaf and its parents open; F02.3.3.4.1 through F02.3.3.4.3 address the separate WGSL
grammar and record-renewal path. No remote CI result, runtime behavior, compatibility, or
performance conclusion is claimed.
