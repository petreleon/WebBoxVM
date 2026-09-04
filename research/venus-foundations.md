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
| Feature negotiation | bit 3 plus `VIRTIO_F_VERSION_1` and `FEATURES_OK` | The command never activates from an unnegotiated feature mask. |
| `blob_mem` | `BLOB_MEM_GUEST` (1) only | Guest pages have defined ownership without a browser-visible host aperture. |
| `blob_flags` | zero | Mappable, shareable, and cross-device claims need host contracts not present here. |
| `blob_id` | zero | A guest-only blob has no host object identifier. |
| Backing entries | 0–16,384 valid physical ranges covering `size` when present | Later attach/detach supports swap-style guest-page lifetime. |
| Context attachment | lifecycle attachment only | A VirGL context may retain the ID, but draw-state validation still rejects it. |
| Lifetime | normal `RESOURCE_UNREF` | IDs share one namespace and accounting budget with 2D and bounded VirGL resources. |

The device neither exposes the host-visible shared-memory selector nor accepts
`BLOB_MEM_HOST3D` / `BLOB_MEM_HOST3D_GUEST`, `RESOURCE_MAP_BLOB`, or
`RESOURCE_UNMAP_BLOB`. It therefore cannot expose a guest pointer to browser
memory and cannot represent Vulkan external-memory ownership.

## Safety invariant

For every live blob ID there is exactly one validated (possibly empty)
guest-backing vector and one accounted logical byte range. No failed request
reserves an ID, consumes a budget byte, or changes an existing resource.
`RESOURCE_UNREF` clears the backing vector before its ID can be reused.
Lookups are expected O(1); validation is O(n) only in the bounded list of `n`
backing entries.

## Why this precedes Venus

Mesa documents Venus as a VirtIO-GPU Vulkan command-serialization path that
requires resource blobs, host-visible memory, and host Vulkan/external-memory
support. The current profile satisfies only the first resource-lifecycle step;
WebGPU does not itself provide the host Vulkan external-memory primitives Venus
needs. [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)

The next truthful layers are:

1. a real host-visible aperture with map/unmap lifetime and cache-mode rules;
2. `HOST3D_GUEST` ownership and synchronization semantics;
3. a Vulkan-capable host boundary, only if it can preserve external-memory and
   fence behavior; and finally
4. a Venus capset whose queried properties match those completed layers.
