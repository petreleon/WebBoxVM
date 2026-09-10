# F02.4.4.1.5.2.3.3.2 — Bind the core input and scope manifest

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.3.2
Depends: F02.4.4.1.5.2.3.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: the [input observer](../01-observe-pinned-build-inputs/README.md),
[actual Docs grammar](../../02-actual-closure-identity/README.md), and
[official build witness](../../01-reproduce-pinned-html/evidence.md).

## Outcome

A fail-closed, unadmitted model turns the observed raw/generated phase records into one bounded core input/scope
manifest. It distinguishes required extension control and promotion metadata from inactive extension semantics.

## Starting points

- [identity parser](../../02-actual-closure-identity/vulkan_docs_identity_parse.py)
- [identity member model](../../02-actual-closure-identity/vulkan_docs_identity_members.py)
- [reviewed Docs scope](../../02-actual-closure-identity/vulkan_docs_identity_scope.py)

## Checklist

- [x] Bind ordered raw immutable identities, derived identities, producer coverage, per-member caps, and exact core
  configuration without treating an output as a source.
- [x] Bind WSI/video false branches, required extension-control inputs, inactive individual extension branches, images,
  and the four core-promotion metadata inputs.
- [x] Reject root-only, omitted, duplicate, malformed, over-limit, unsafe-path, stale-producer, WSI/video, or
  extension-expanded manifests before exposing a result.
- [x] Add focused positive and hostile fixtures that keep all public results unadmitted and not cutover-ready.

## Verification

- The V1 extension directive remains an audit boundary; it does not deny its required control source.
- This grammar validates capture identity and scope only; it does not stage cache payloads or prove a complete closure.
