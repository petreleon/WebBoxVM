# Venus resource foundations

## Scope boundary

WebBoxVM implements a deliberately small resource-blob subset: negotiated
`BLOB_MEM_GUEST`, a bounded mapped `BLOB_MEM_HOST3D` staging object, and a
`BLOB_MEM_HOST3D_GUEST` default blob with explicit guest-shadow transfers. This
is not a Venus capset, a Vulkan implementation, or a promise that Mesa can
create a Vulkan device.

The precise nonzero-`blob_id` ordering boundary is recorded in
[renderer-local blob ordering](renderer-blob-ordering.md).

The current Linux wire ABI assigns `VIRTIO_GPU_F_RESOURCE_BLOB` to feature bit
3 and `RESOURCE_CREATE_BLOB` to control command `0x010c`. The command carries
a resource ID, memory kind, flags, backing-entry count, blob ID, size, and a
trailing list of guest physical-memory entries. [Linux UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)

## Implemented profile

| ABI field | Accepted value | Reason |
| --- | --- | --- |
| Feature negotiation | bit 3 plus `VIRTIO_F_VERSION_1` and `FEATURES_OK`; bit 5 when selected | Blob creation never activates from an unnegotiated resource-blob feature mask. |
| Guest `blob_mem` / flags | `BLOB_MEM_GUEST` (1), zero flags | Guest pages retain their defined ownership. |
| Host staging `blob_mem` / flags | `BLOB_MEM_HOST3D` (2), `BLOB_FLAG_USE_MAPPABLE` | Creation requires a live VirGL context and creates one bounded CPU-visible staging allocation. |
| Default shadow `blob_mem` / flags | `BLOB_MEM_HOST3D_GUEST` (3), zero flags | Creation records a live VirGL context and retains host bytes plus an explicitly synchronized guest shadow. |
| `blob_id` | zero for legacy profiles; nonzero after `WBL1` preparation | The nonzero path is context-local and consumes its preparation record once. |
| Guest backing entries | 0–16,384 valid physical ranges covering `size` when present | Guest/default blobs may attach or detach their shadow pages later. |
| Host staging backing entries | zero | The profile retains its bounded bytes internally, without a guest shadow buffer. |
| Context association | creation-time owner plus lifecycle attachment | Default 3D transfers require their recorded capset-1 owner; draw-state validation still rejects blobs. |
| Lifetime | normal `RESOURCE_UNREF` | IDs share one namespace and accounting budget with 2D and bounded VirGL resources. |

The device also implements a tightly scoped host-visible map profile. The
VirtIO-MMIO shared-memory selector exposes ID 1 at a 64 MiB sparse aperture
starting at guest physical `0x0b00_0000`. `BLOB_MEM_HOST3D` is accepted only
for a 4 KiB-aligned, mappable, backing-free blob created in a live VirGL
context. `RESOURCE_MAP_BLOB` assigns a page-aligned offset, returns cached map
info, restores that blob's retained bytes into the aperture, and rejects an
overlap or a duplicate mapping. `RESOURCE_UNMAP_BLOB` copies the aperture back
to the bounded host allocation and discards the sparse pages. Unref and device
reset also discard live mappings, preventing data leakage on aperture reuse.

Default blobs are deliberately not aperture-mapped: map/unmap remains exclusive
to the host-only profile. With valid guest backing installed, a 3D
transfer-to-host copies a bounded range into retained host bytes, and the
inverse transfer restores that range to guest memory. An unattached or detached
default shadow rejects either transfer without changing retained host bytes.

## Context-local allocation ordering

For a nonzero `blob_id`, the capset-1 context must first submit an exact 32-byte
private `WBL1` preparation envelope. It records the ID, size, blob-memory kind,
and flags without allocating bytes. The following matching `RESOURCE_CREATE_BLOB`
consumes that record only after all resource and backing checks succeed. Mismatched
or failed creates preserve it for retry; it is single-use after success and is
dropped with the context. The ledger is bounded to 64 records per context.

This mirrors the required `SUBMIT_3D`-before-create ordering, but `WBL1` is a
WebBoxVM transport probe, not a Venus command or renderer implementation. Mesa
and a real Venus renderer do not interoperate with it.

It does not accept shareable/cross-device flags, a real Venus renderer
allocation command stream, or any Vulkan external-memory handle. The profiles
have no GPU command interpretation, fence export, or Vulkan ownership transfer.

## Safety invariant

For every live blob ID there is exactly one validated (possibly empty)
guest-backing vector and one accounted logical byte range. No failed request
reserves an ID, consumes a budget byte, or changes an existing resource.
`RESOURCE_UNREF` clears the backing vector before its ID can be reused.
Lookups are expected O(1); validation is O(n) only in the bounded list of `n`
backing entries.

For mapped host blobs, the aperture intervals are pairwise disjoint and every
interval has exactly one live owner. A map copies retained bytes into the
aperture; unmap performs the inverse copy before dropping sparse pages. Thus a
mapping round trip preserves bytes while an ID's release makes its aperture
range read as zero.

Default-shadow transfers require level/stride zero, a nonempty bounded range,
and complete validated backing. They copy only that range between the retained
host shadow and guest memory; a rejected request leaves the host shadow intact.
Prepared renderer-object records are context-scoped, metadata-exact, bounded,
and cannot be consumed by a failed allocation or another context.

## Why this precedes Venus

Mesa documents Venus as a VirtIO-GPU Vulkan command-serialization path that
requires resource blobs, host-visible memory, and host Vulkan/external-memory
support. The current profile supplies guest ownership, host staging, and
default-shadow coherence, but WebGPU does not itself provide the host Vulkan
external-memory primitives Venus needs. [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)

The next truthful layers are:

1. replace the private preparation envelope with the real Venus renderer-object
   command protocol and its nonzero `blob_id` allocation;
2. a Vulkan-capable host boundary, only if it can preserve external-memory and
   fence behavior; and finally
3. a Venus capset whose queried properties match those completed layers.
