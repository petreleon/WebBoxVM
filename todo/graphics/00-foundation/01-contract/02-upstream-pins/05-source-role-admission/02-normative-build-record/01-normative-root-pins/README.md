# F02.5.2.1 — Pin and re-fetch normative roots

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.2.1
Depends: F02.5.1
Evidence: pending

## Outcome

OpenGL 4.6, GLES 3.2, Vulkan-Docs prose root, and `vk.xml` have immutable Khronos records with
their exact bytes, digest, terms, attribution, and an external-cache re-fetch verifier.

## Starting points

- [reviewed candidates](../../../04-profile-source-inputs/candidate_catalog.py)
- [shared role contract](../../01-authority-and-transform-boundary/source_role_contract.py)
- [external fetch contract](../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)

## Checklist

- [ ] Keep each root a distinct upstream-source record; do not silently substitute a registry or PDF.
- [ ] Re-fetch into an explicit external cache and fail closed on URL, byte, digest, or cache-path mismatch.
- [ ] Reject mutable, partial, unlicensed, misattributed, or locally relabeled source records.
- [ ] Record the focused command and fresh-fetch result in a receipt.

## Verification

- A fresh cache can contain exactly the four pinned payloads only when every declared identity matches.
- This task proves source identity only, not API support, conformance, certification, profile support,
  performance, or a Khronos-authored local artifact.
