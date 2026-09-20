# F03.2.2.1 — Verify the normative PDF cache

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.2.1
Depends: F03.2.1
Evidence: pending

## Outcome

One explicit external-cache reader accepts only the exact F03.2.1 OpenGL 4.6 core normative PDF before
any extractor sees bytes. It proves source identity, not API behavior or a matrix row.

## Starting points

- [source-authority boundary](../../01-source-authority/README.md)
- [F03 role-aware binding](../../../01-profile-scope/role_aware_bindings.py)
- [F02 external-cache contract](../../../../02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [ ] Load F03.2.1 only through its fixed private path and accept the `command-object-state` decision.
- [ ] Require an absolute external nonsymlink cache root and its exact identity-derived PDF path.
- [ ] Open the PDF without following links, then verify its exact byte count and SHA-256 before parsing.
- [ ] Reject a missing, stale, mixed, in-repository, symlinked, oversized, or wrong-profile cache payload.
- [ ] Preserve the physical one-based PDF page count for downstream locator validation without treating it
  as a support claim.
- [ ] Add focused cache-boundary tests and attach a no-claim receipt.

## Verification

The cache supplies only the sealed normative source bytes. It does not admit `gl.xml`, GLSL, extensions,
CTS cases, a compatibility profile, or a runtime result.
