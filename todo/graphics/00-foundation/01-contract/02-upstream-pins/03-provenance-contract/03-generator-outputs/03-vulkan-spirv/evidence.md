# F02.3.3.3 evidence

Revision: `ccbe47aadd5284d668656591b54f3b5db84aa437` baseline; scoped feature diff pending commit
Validation: hermetic record, generator-output, inventory, source-limit, full-suite, and roadmap checks
Result: PASS
Artifacts: two local provenance-only fixtures and JSON sidecars; no upstream payload bytes
Profile: future-generator contract fixture only; no Vulkan, SPIR-V, Venus, guest, or runtime behavior

## Bound identities and outputs

Both records name F02.1 raw-manifest SHA-256
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e` and generator
`webboxvm-provenance-fixture-generator` version `1`.

| Fixture | Sole generator input | Fixture SHA-256 |
| --- | --- | --- |
| `fixture/vulkan-registry.fixture` | `vulkan-registry` / `cf31c965...ffd9da06` | `fe60ddd8...35d0ad12` |
| `fixture/spirv-core-grammar.fixture` | `spirv-core-grammar` / `db858127...fe660ba4` | `542b5d75...296866a1a` |

The sidecars also retain each exact F02.1 license, command, artifact path, and complete output hash.
The generator derives markers only from those declared identities and never reads a registry or grammar
payload; committed fixtures contain no upstream source bytes.

## Commands and actual results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/03-vulkan-spirv/validate_vulkan_spirv.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/01-provenance-record/provenance_record_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

The focused Vulkan/SPIR-V suite passed 5/5: both records resolve and reproduce exactly; a wrong
generator version, cross-family SPIR-V input, `vk-gl-cts-api-version`, `mesa-venus-device` runtime
input, stale digest, and wrong artifact hash all fail offline. F02.3.1's contract suite passed 9/9,
F02.1's inventory suite 4/4, and source limits 6/6. Full-suite and final structural results are
recorded after the feature commit.

No graphics API, generated protocol code, browser execution, Mesa/Venus integration, or performance
result is claimed. Next independent tasks are the remaining F02.3.3 family children and F02.3.2;
F02.3.3.4 remains open because its current inventory inputs are reference-only.
