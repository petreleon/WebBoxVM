# F02.4.4.1.5.2.3 progress record — actual Docs closure

Revision: `5ba2a53fc52cd31892be9ff562ff4622c6c0b784` clean tested baseline
Validation: two fresh official core HTML builds and the completed reproduction child
Result: IN PROGRESS
Artifacts: ignored `.artifacts/vulkan-docs-official.{NY6qhJ,K5cEx0}` and
`.artifacts/vulkan-docs-build.{SfExfa,nkp7Vk}`; no Docs payload is stored in Git
Profile: successor-only source-closure work; no admission, guest API, browser, CTS, conformance, or performance claim

The official pinned build route now has a successful, byte-identical two-run receipt in
[F02.4.4.1.5.2.3.1](01-reproduce-pinned-html/evidence.md). It supersedes the earlier login-shell `pyparsing`
preflight error without installing or substituting a package.

The actual output `out/html/vkspec.html` is 10,377,052 B. That is deliberately recorded as a rendered-output
identity, not misclassified as an F02.2 `SourceInput`: F02.2's 8 MiB limit protects immutable fetched source
members, while the future post-cutover grammar concerns closure members rather than rendered artifacts. The synthetic
fixture's output/member conflation cannot be reused for this real closure. The next child owns a separate, unadmitted
actual-Docs grammar with an explicit bounded output-artifact policy, while retaining the hard F02.2 limit for raw and
generated closure inputs.

No current closure blocker is claimed by this progress record. The active work is to make that distinction
fail-closed, capture the actual core scope and inputs, then rehash and compare both build witnesses before any proof.
The independent VCTS selector blocker remains visible and neither lane narrows the final target.
