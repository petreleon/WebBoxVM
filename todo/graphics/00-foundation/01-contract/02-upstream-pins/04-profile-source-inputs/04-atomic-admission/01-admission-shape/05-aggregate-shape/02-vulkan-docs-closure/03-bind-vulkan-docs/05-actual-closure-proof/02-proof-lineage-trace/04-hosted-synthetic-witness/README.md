# F02.4.4.1.5.2.3.5.2.4 — Witness the synthetic collector on hosted Docker

[Parent task](../README.md)

Task: F02.4.4.1.5.2.3.5.2.4
Depends: F02.4.4.1.5.2.3.5.2.3
Evidence: pending

## Outcome

The synthetic collector runs once on a GitHub-hosted x64 runner from immutable Git blobs and emits a strictly
unadmitted, self-validating fixture receipt under the already validated pinned-image Docker policy.

## Starting points

- [hosted primitive witness](../lineage_github_docker_witness.py)
- [synthetic collector child](../03-synthetic-syscall-collector/README.md)
- [proof receipt](../lineage_contract.py)

## Checklist

- [ ] Anchor every executed helper, C source, and fixture byte to the workflow commit before Docker starts.
- [ ] Preserve pinned image, `--pull=never`, no network, user `501:20`, and no privilege/security relaxations.
- [ ] Require an exact collector terminal receipt and reject a runtime or malformed trace as workflow failure.
- [ ] Record the public run separately from local tests without promoting the fixture into Docs lineage evidence.

## Verification

- The remote receipt must prove only the self-contained fixture and explicitly retain Docker/runner trust limits.
- A green workflow without a valid terminal receipt is a failure, never a capability result.
