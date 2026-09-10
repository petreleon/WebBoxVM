# F02.4.4.1.5.4.4.2.4 — Rehash and reconcile the successor closure

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.2.4
Depends: F02.4.4.1.5.4.4.2.2, F02.4.4.1.5.4.4.2.3
Evidence: pending

Prerequisite lists: [derived-member authority](../02-derived-member-authority/README.md),
[authorized write lineage](../03-authorized-write-lineage/README.md), and the
[F02 fetch contract](../../../../../../../../02-fetch-verifier/README.md).

## Outcome

One later reconciliation rehashes all policy-valid raw inputs in an isolated external cache, compares the
two fresh closures, preserves core/WSI/video/extension treatment, and records only an unadmitted Docs
source-role result.

## Starting points

- [source policy](../../01-derived-docs-policy/README.md)
- [historical anchor](../01-successor-closure-anchor/README.md)
- [active F02 cache policy](../../../../../../../../02-fetch-verifier/README.md)

## Checklist

- [ ] Rehash every raw member under the 8 MiB cap in an isolated external cache with safe names.
- [ ] Compare both authorized closure manifests, scope, producer lineage, and rendered output identities.
- [ ] Preserve exact core, WSI, video, extension, and unknown treatment without selector filtering.
- [ ] Reject mutation, omission, alias, symlink, cache race, stale run, root-only, and output-as-source input.
- [ ] Keep aggregate admission, inventory cutover, cache freshness, F03, support, release, and performance false.
- [ ] Attach a complete hostile-tested unadmitted closure receipt.

## Verification

- Only this child may state that the successor Docs source role is proven; it still cannot admit the
  aggregate or change the independent GLES/VCTS obligations.
