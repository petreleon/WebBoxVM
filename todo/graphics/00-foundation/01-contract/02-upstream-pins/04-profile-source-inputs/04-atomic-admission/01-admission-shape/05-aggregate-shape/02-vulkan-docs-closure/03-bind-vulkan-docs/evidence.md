# F02.4.4.1.5.2.3 progress record — actual Docs closure

Revision: `af343ce3024a778f9d8ef89616d93699619981d8` verified Docs-identity grammar
Validation: completed two-build witness and F02.4.4.1.5.2.3.2's 29 focused grammar tests
Result: IN PROGRESS
Artifacts: ignored `.artifacts/vulkan-docs-official.{NY6qhJ,K5cEx0}` and
`.artifacts/vulkan-docs-build.{SfExfa,nkp7Vk}`; no Docs payload is stored in Git
Profile: successor-only source-closure work; no admission, guest API, browser, CTS, conformance, or performance claim

The official pinned build route now has a successful, byte-identical two-run receipt in
[F02.4.4.1.5.2.3.1](01-reproduce-pinned-html/evidence.md). It supersedes the earlier login-shell `pyparsing`
preflight error without installing or substituting a package.

The actual output `out/html/vkspec.html` is 10,377,052 B. It is now bound as a rendered-output identity, not
misclassified as an F02.2 `SourceInput`: F02.2 keeps its 8 MiB cap for raw and generated closure members, while this
unadmitted grammar has separate 16 MiB per-output, 32 MiB tree, and 4,096-file bounds. The pinned image, source root,
recipe, toolchain, two equal tree hashes, output producer, and predecessor's rejected state are all fail-closed.

This completion is intentionally only the grammar/witness child. It does not claim the complete resolved conditional or
promotion-metadata selector set; child `.3` must capture that real input/scope closure, then `.4` stages its full tree
manifest and `.5` proves it. The independent VCTS selector blocker remains visible and neither lane narrows the target.
