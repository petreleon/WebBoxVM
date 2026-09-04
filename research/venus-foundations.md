# Venus resource foundations

## Scope boundary

WebBoxVM now implements one real, deliberately small prerequisite from the
VirtIO-GPU resource-blob ABI: a negotiated `BLOB_MEM_GUEST` resource. This is
not a Venus capset, a Vulkan implementation, or a promise that Mesa can create
a Vulkan device.

The current Linux wire ABI assigns `VIRTIO_GPU_F_RESOURCE_BLOB` to feature bit
3 and `RESOURCE_CREATE_BLOB` to control command `0x010c`. The command carries
a resource ID, memory kind, flags, backing-entry count, blob ID, size, and a
trailing list of guest physical-memory entries. [Linux UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)

## Implemented profile

| ABI field | Accepted value | Reason |
| --- | --- | --- |
| Feature negotiation | bit 3 plus `VIRTIO_F_VERSION_1` and `FEATURES_OK`; bit 5 when selected | Blob creation never activates from an unnegotiated resource-blob feature mask. |
| Guest `blob_mem` / flags | `BLOB_MEM_GUEST` (1), zero flags | Guest pages retain their defined ownership. |
| Host staging `blob_mem` / flags | `BLOB_MEM_HOST3D` (2), `BLOB_FLAG_USE_MAPPABLE` | A live VirGL context owns one bounded CPU-visible staging allocation. |
| `blob_id` | zero | Neither profile exposes an external or renderer-local object identity. |
| Guest backing entries | 0–16,384 valid physical ranges covering `size` when present | Later attach/detach supports swap-style guest-page lifetime. |
| Host staging backing entries | zero | The profile retains its bounded bytes internally, without a guest shadow buffer. |
| Context association | lifecycle attachment only | A live VirGL context owns host staging lifetime, but draw-state validation still rejects it. |
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

It does not accept `BLOB_MEM_HOST3D_GUEST`, shareable/cross-device flags, or
any Vulkan external-memory handle. The mappable `HOST3D` profile is a
CPU-visible staging allocation: it has no renderer-local object validation,
GPU command interpretation, fence export, or Vulkan ownership transfer.

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

## Why this precedes Venus

Mesa documents Venus as a VirtIO-GPU Vulkan command-serialization path that
requires resource blobs, host-visible memory, and host Vulkan/external-memory
support. The current profile now supplies the first two resource-lifecycle
layers, but WebGPU does not itself provide the host Vulkan external-memory
primitives Venus needs. [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)

The next truthful layers are:

1. `HOST3D_GUEST` ownership and synchronization semantics;
2. a Vulkan-capable host boundary, only if it can preserve external-memory and
   fence behavior; and finally
3. a Venus capset whose queried properties match those completed layers.
