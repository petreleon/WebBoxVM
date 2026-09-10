# F02.4.4.1.5.2.3.5.2.2 — Validate the bounded ptrace capability primitive

[Parent task](../README.md)

Task: F02.4.4.1.5.2.3.5.2.2
Depends: F02.4.4.1.5.2.3.3.2, F02.4.4.1.5.2.3.3.3
Evidence: [receipt](evidence.md)

## Outcome

A Git-blob-anchored C primitive verifies only parent/child syscall, fork, and exec tracing under the pinned image and
default hosted-Docker policy, without claiming that a Docs collector is available.

## Starting points

- [C primitive](../observer/lineage_ptrace_probe.c)
- [hosted Docker witness](../lineage_github_docker_witness.py)
- [policy tests](../lineage_github_docker_witness_test.py)

## Checklist

- [x] Require immutable Git blobs, a clean anchored commit, and a reviewed source-envelope hash/byte count.
- [x] Require the pinned linux/amd64 image and run with `--pull=never`, `--network none`, and user `501:20`.
- [x] Refuse runtime, compiler, schema, image, envelope, and terminal-record failures rather than issue a green receipt.
- [x] Record the hosted result only as `observed-unadmitted`, without a Docs mount, build, lineage, or closure claim.

## Verification

- The local tests assert anchored inputs, exact image policy, tamper rejection, and no privilege relaxation.
- A successful primitive is not authority for collector tracing, Docs provenance, or closure.
