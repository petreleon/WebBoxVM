# F03.2.3 — Extract limit, format, shader, and extension rows

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.3
Depends: F03.2.1
Evidence: pending

## Outcome

The complementary OpenGL 4.6 core limits, formats, shader rules, and extension decisions are explicit,
source-located matrix facts. An extension remains outside the target until a recorded decision admits it.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [F03 v2 profile schema](../../01-profile-scope/profile_contract_v2.py)
- [OpenGL parent task](../README.md)

## Checklist

- [ ] Consume only F03.2.1-accepted locator classes for numeric limits, formats, shader semantics, and
  extension decisions; reject a distinct unadmitted registry or language document.
- [ ] Enumerate every mandatory core limit, format property, shader rule, and version constraint in the
  shared row schema with its exact source locator.
- [ ] Record each extension as adopted, excluded, or unresolved with a reason; keep compatibility-profile
  behavior and lower-version substitutions outside the target.
- [ ] Distinguish a required property from an optional capability, implementation observation, or future
  feature so a missing item cannot disappear from coverage.
- [ ] Reject missing, duplicate, stale, profile-mismatched, and falsely supported rows through focused
  negative checks.
- [ ] Attach a no-claim inventory receipt and keep the OpenGL profile `matrix-incomplete`.

## Verification

This leaf fixes inventory facts and scope decisions only. It does not prove shader execution, format
support, device behavior, CTS execution, a browser path, certification, or performance.
