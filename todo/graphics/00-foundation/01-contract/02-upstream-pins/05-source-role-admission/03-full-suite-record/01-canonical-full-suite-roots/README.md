# F02.5.3.1 — Pin canonical full-suite roots

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.3.1
Depends: F02.5.1
Evidence: [receipt](evidence.md)

## Outcome

The three profile records name only unmodified Khronos full-suite roots: OpenGL 4.6
gl46-main, GLES 3.2 mustpass.xml, and the released Vulkan vk-default.txt. Each record
has immutable revision, digest, bytes, terms, source cache key, and full-suite-root
role; only its Khronos selector field may be true.

## Starting points

- [candidate catalog](../../../04-profile-source-inputs/candidate_catalog.py)
- [source-role policy](../../01-authority-and-transform-boundary/source-role-policy.md)

## Checklist

- [x] Re-fetch each immutable root into an empty cache and verify exact bytes and digest.
- [x] Bind the two OpenGL-family roots to 067e8832315e79817ede1c4863804e440f5d1c80.
- [x] Bind Vulkan vk-default.txt to release vulkan-cts-1.4.6.2 at f6a29701220f34dd1407513bfe80d74ca7b392ce.
- [x] Reject mutable revisions, wrong paths, false unfiltered status, and non-Khronos root roles.
- [x] Record that Vulkan vk-default is a full default root, not a Khronos Vulkan-1.4-core-only selector.

## Verification

Each root is fetched from its immutable URL, re-hashed from cache, has Khronos authority
and producer, is unfiltered, and makes no API support, conformance, certification,
profile, or performance claim.
