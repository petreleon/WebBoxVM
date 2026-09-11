# F02.5.3.4 — Build local shards and publish a no-claim receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.3.4
Depends: F02.5.2.2, F02.5.3.2, F02.5.3.3
Evidence: pending

## Outcome

An explicitly WebBoxVM-produced byte-preserving shard mode splits selected upstream
members into artifacts no larger than 8 MiB, proves exact reassembly, and publishes a
receipt that keeps every support and qualification claim false.

## Starting points

- [local Vulkan builder](../../02-normative-build-record/02-vulkan-local-definition/webboxvm_source_builder.py)
- [shared artifact verifier](../../01-authority-and-transform-boundary/source_role_artifacts.py)

## Checklist

- [ ] Add and pin a separate byte-preserving-shard builder mode; do not relabel core-definition mode.
- [ ] Exercise the real Vulkan api.txt member, 40,296,059 bytes, in five bounded shards.
- [ ] Bind offsets, source identity, shard identity, and exact reassembled SHA-256.
- [ ] Reject oversize, missing, reordered, altered, or root-mismatched shards and altered builder bytes.
- [ ] Publish a receipt with zero CTS executions and false API support, conformance, certification, profile, and performance claims.

## Verification

The 8 MiB requirement applies to the local shard artifacts only. Shards remain
WebBoxVM transforms and never become a Khronos selector or substitute for the entire
unfiltered suite.
