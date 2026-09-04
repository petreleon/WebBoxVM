# Research: renderer-local blob ordering

## Question

How can WebBoxVM validate the VirtIO-GPU ordering requirement for a nonzero
host-3D `blob_id` before it has a Venus renderer?

## Primary-source findings

VirtIO requires `HOST3D` and `HOST3D_GUEST` resources to be created from a
context-local object identified by `blob_id`; allocation occurs through
`SUBMIT_3D`. The guest shadow of a default blob may initially have zero backing
entries and receive them later. [VirtIO GPU specification](https://github.com/oasis-tcs/virtio-spec/blob/master/device-types/gpu/description.tex)

Linux accepts a dword-aligned `cmd_size` for host-3D blobs, preserves the file
context ID and `blob_id`, queues `virtio_gpu_cmd_submit`, then creates the GEM
object. It rejects a command or nonzero ID for a guest-only blob.
[Linux VirtIO-GPU ioctl path](https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/virtio/virtgpu_ioctl.c)

## WebBoxVM transport contract

`WBL1` is an exact 32-byte private opaque payload for a capset-1 context:

```text
WBL1 | version=1 | blob_id:u64 | size:u64 | blob_mem:u32 | blob_flags:u32
```

It creates no resource or host allocation. Instead, each context records at
most 64 prepared objects in a hash map. A later host-3D/default blob with a
nonzero ID must match all four fields. Successful creation consumes the record;
failed creation preserves it; context destruction drops every record.

## Invariant and cost

For each live record `(context, blob_id)` is unique. A create may consume a
record only after every resource, feature, context, size, and backing check
passes. Thus a rejected request cannot make a later mismatched create valid.
Lookup, insert, and consume are expected `O(1)`; each context retains at most
64 small records and no prepared record allocates the requested blob bytes.

## Boundary

`WBL1` proves Linux-to-device ordering and context scoping only. It is not a
VirGL command, Venus protocol command, Vulkan object, capset, external-memory
handle, or fence. A real Venus path must replace it with the generated Venus
renderer protocol and a Vulkan/external-memory-capable host boundary.

## Validation

Rust tests cover absent, mismatched, one-shot, retry, cross-context, and
context-destruction behavior. The freestanding AArch64 guest sends a genuine
Linux `RESOURCE_CREATE_BLOB` ioctl with `cmd_size=32` and a nonzero ID; a fresh
Linux VM reaches its PASS marker only when the device accepts the ordered pair.
