# F03.3.2.1 — Verify the normative PDF cache

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.3.2.1
Depends: F03.3.1
Evidence: [receipt](evidence.md)

## Outcome

One explicit external-cache reader accepts only the exact F03.3.1 GLES 3.2 normative PDF before either
admitted extractor sees bytes. It proves source identity, not API behavior or a matrix row.

## Starting points

- [source-authority boundary](../../01-source-authority/README.md)
- [F03 role-aware binding](../../../01-profile-scope/role_aware_bindings.py)
- [F02 external-cache contract](../../../../02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] Load F03.3.1 only through its fixed private path and accept `command-state` and `limit-format`.
- [x] Require an absolute external nonsymlink cache root and its exact identity-derived PDF path.
- [x] Open the PDF without following links, then verify its exact byte count and SHA-256 before parsing.
- [x] Reject a missing, stale, mixed, in-repository, symlinked, oversized, or wrong-profile cache payload.
- [x] Preserve physical one-based PDF page bounds for downstream locator validation without claiming support.
- [x] Add focused cache-boundary checks and attach a no-claim receipt.

## Verification

The cache supplies only sealed normative source bytes. It does not admit ESSL, `gl.xml`, extensions, CTS
cases, a compatibility profile, or a runtime result.
