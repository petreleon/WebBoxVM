# F03.4.2.2 — Pin citation-only raw Docs locators

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.4.2.2
Depends: F03.4.2.1
Evidence: pending

## Outcome

A WebBoxVM-owned, citation-only map binds reviewed raw Vulkan-Docs fragments to the admitted normative
root. It records exact raw bytes and semantic anchors without promoting fragments, generated output, or
the map itself into a new source role.

## Starting points

- [source-channel boundary](../01-source-channel-boundary/README.md)
- [sealed F02 source contract](../../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [raw registry inventory](../../01-registry-inventory/README.md)

## Checklist

- [ ] Refresh a new external cache of the reviewed raw Docs fragments at the admitted revision; never use generated Docs.
- [ ] Bind each accepted citation to its root identity, raw path, digest, bytes, license/attribution, and one exact semantic anchor.
- [ ] Keep the map citation-only: it is neither a full rendered Docs closure nor an independent normative root.
- [ ] Leave an unreviewed or ambiguous registry fact blocked rather than inferring a symbol-name locator.
- [ ] Reject mixed revisions, bad licenses, missing/ambiguous anchors, cache escapes, generated paths, and role promotion.
- [ ] Attach a bounded no-claim map receipt; keep full row data external when it is too large for a readable source file.

## Verification

Raw citation provenance may support later planning only. It cannot create a matrix row, imply source
coverage, or make API, guest, CTS, certification, browser, or performance claims.
