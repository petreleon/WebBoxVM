# Hermetic Git-tree plan contract

`vcts_tree_plan.py` makes no network request. Its `build(api_input, identity_path)` API accepts only
a normalized, caller-supplied Git-tree metadata object and returns a self-hashed plan. It rejects any
input that is not anchored to the checked canonical V2 identity, including the exact annotated-tag
object and peeled commit.

The input must supply the tag target, commit tree, and the five parent-linked named trees
`external/vulkancts/mustpass/main/vk-default`; it also binds the selector root to `main` and the member
catalogue to `vk-default`. It records the selector-root blob (`100644`, 3,347 bytes) and the 98 ordered
root-selector paths. Every member is
an exact `{path, mode, blob_sha1, bytes}` record; only `100644` regular files within the fixed limits are
accepted. The generated plan also records its member count, aggregate bytes, V2 identity digest, and
canonical JSON SHA-256.

`validate(plan_path, identity_path)` replays those checks and returns a small `TreePlan` summary. It
rejects duplicate keys, stale aggregates or digest, missing/extra/reordered members, unsafe paths,
substituted tag/commit data, malformed tree links, and per-member or aggregate limit excesses.

This is an unpopulated planning boundary, not a claim that the supplied metadata came from Khronos or
that any payload was fetched. The later live-capture worker must obtain and authenticate the API responses,
stream raw bytes, verify their Git blob SHA-1 values, and prove offline cache rehashing before the task can
be marked complete.
