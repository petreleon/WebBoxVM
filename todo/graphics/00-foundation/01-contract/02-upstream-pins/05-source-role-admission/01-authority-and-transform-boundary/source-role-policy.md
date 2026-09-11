# F02.5 source-role policy

## Invariant

`source provenance != WebBoxVM derivation authority != core coverage mapping != full CTS execution
!= Khronos certification != browser compatibility != near-native performance`.

## Roles

| Role | Authority and producer | Permitted statement |
| --- | --- | --- |
| `upstream-source` | Khronos / Khronos | A pinned raw normative source or suite member exists. |
| `webboxvm-transform` | WebBoxVM / WebBoxVM | A pinned builder identity and canonical argv name exact catalogued inputs. |
| `full-suite-root` | Khronos / Khronos | A pinned canonical unfiltered selector exists. |

The catalog resolves every transform input by ID, SHA-256, kind, and upstream revision, then rejects
cycles. Upstream records have an immutable URL, commit, digest, byte count, license, attribution,
scope, and a safe external-cache key. Local transforms have an output key/digest, sealed builder
identity, input identities, and argv; they do not pretend to have a Khronos URL or commit. The argv
is a fixed `webboxvm-source-builder` recipe: it has no shell or URL token and binds every declared
input and exactly one output token. F02.5.2 must audit and pin the actual builder/toolchain before
recording a real transform.

`validate_catalog` proves record and graph structure only. A consumer must call
`verify_catalog` with the external artifact root before accepting any record: it streams every
artifact and builder, recomputes bytes/SHA-256, and rejects declared-small or tampered content.

## Claim boundary

Claims are closed: `api_support`, `conformance`, `certification`, `profile_support`, and `performance`
are false for every role. Only a `full-suite-root` has `khronos_selector=true`; that means locator
authority, not that any CTS case ran. F02.5.3 must fetch and receipt the exact released root before
it can establish unmodified full-suite qualification. A local map, document, or shard always has all
claims false.

`vulkan-1.4-core` names the guest API target. Its canonical final root is the unfiltered VCTS
`vk-default.txt`, including WSI, video, and extensions; it must never be relabeled a Khronos
Vulkan-1.4-core-only selector.

## Size boundary

The 8 MiB cap applies only to WebBoxVM-produced transforms and shards, never to an upstream raw
source or full-suite root. Every shard records its whole upstream suite member, SHA-256, byte count,
piece index/count/offset, reassembly digest, and its matching pinned full-suite root. The catalog
requires complete ordered byte coverage; `verify_catalog` streams and compares actual reassembly to
the immutable member. F02.5.3 must additionally check that member against the root's ordered ledger.
