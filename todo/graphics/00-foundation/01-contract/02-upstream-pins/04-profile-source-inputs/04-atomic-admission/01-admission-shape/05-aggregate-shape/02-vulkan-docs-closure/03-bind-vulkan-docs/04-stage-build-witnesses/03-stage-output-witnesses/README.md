# F02.4.4.1.5.2.3.4.3 — Stage both bounded output witnesses

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4.3
Depends: F02.4.4.1.5.2.3.4.1, F02.4.4.1.5.2.3.4.2
Evidence: [receipt](evidence.md)

Prerequisite lists: the [staging contract](../01-stage-contract/README.md), [staged core inputs](../02-stage-core-inputs/README.md),
and [actual build witness](../../02-actual-closure-identity/README.md).

## Starting points

- [staging-contract task](../01-stage-contract/README.md)
- [input-staging task](../02-stage-core-inputs/README.md)
- [reviewed build witness](../../02-actual-closure-identity/vulkan_docs_build_witness.json)

## Outcome

Descriptor-walk, rehash, and separately stage both full generated-tree witnesses under the reviewed build context;
neither equal content nor an inherited tree digest may collapse their two physical recorded-run origins.

## Checklist

- [x] Bind the reviewed image, platform, argv, environment, mounts, recipe/toolchain, source identities, and both clean-run identities.
- [x] Enforce per-tree bounds of 4,096 regular files and 32 MiB while rehashing exactly 2,530 files and 17,019,466 bytes per selected tree.
- [x] Require tree SHA-256 `26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32` and primary `out/html/vkspec.html` at 10,377,052 bytes with SHA-256 `896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041`.
- [x] Reject unsafe paths, symlinks/non-regular files, missing/extra/duplicate members, partial trees, altered primary HTML, aliasing, output/source substitution, or witness mutations.
- [x] Add positive and hostile output-tree, build-identity, bounds, and descriptor-filesystem regressions with a compact receipt.

## Verification

- This child stages output evidence only under the isolated namespace and does not publish a reusable completion marker.
- It records evidence of captured output witnesses, not a fresh official build or compatibility result.
