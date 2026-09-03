#ifndef VIRGL_CLEAR_DEMO_UAPI_H
#define VIRGL_CLEAR_DEMO_UAPI_H

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef signed int s32;
typedef unsigned long u64;

/* asm-generic Linux ioctl encoding used by AArch64. */
#define IOC_NRBITS 8u
#define IOC_TYPEBITS 8u
#define IOC_SIZEBITS 14u
#define IOC_NRSHIFT 0u
#define IOC_TYPESHIFT (IOC_NRSHIFT + IOC_NRBITS)
#define IOC_SIZESHIFT (IOC_TYPESHIFT + IOC_TYPEBITS)
#define IOC_DIRSHIFT (IOC_SIZESHIFT + IOC_SIZEBITS)
#define IOC_WRITE 1u
#define IOC_READ 2u
#define IOC(dir, type, nr, size) \
    (((dir) << IOC_DIRSHIFT) | ((type) << IOC_TYPESHIFT) | \
     ((nr) << IOC_NRSHIFT) | ((size) << IOC_SIZESHIFT))
#define DRM_IOWR(nr, type) IOC(IOC_READ | IOC_WRITE, 'd', nr, sizeof(type))
#define DRM_COMMAND_BASE 0x40u

#define VIRTGPU_DRM_CAPSET_VIRGL 1u
#define VIRTGPU_CONTEXT_PARAM_CAPSET_ID 1u
#define VIRTGPU_EXECBUF_FENCE_FD_OUT 0x02u

struct drm_virtgpu_context_set_param {
    u64 param;
    u64 value;
};

struct drm_virtgpu_context_init {
    u32 num_params;
    u32 pad;
    u64 ctx_set_params;
};

struct drm_virtgpu_map {
    u64 offset;
    u32 handle;
    u32 pad;
};

struct drm_virtgpu_resource_create {
    u32 target;
    u32 format;
    u32 bind;
    u32 width;
    u32 height;
    u32 depth;
    u32 array_size;
    u32 last_level;
    u32 nr_samples;
    u32 flags;
    u32 bo_handle;
    u32 res_handle;
    u32 size;
    u32 stride;
};

struct drm_virtgpu_3d_box {
    u32 x;
    u32 y;
    u32 z;
    u32 w;
    u32 h;
    u32 d;
};

struct drm_virtgpu_3d_transfer_to_host {
    u32 bo_handle;
    struct drm_virtgpu_3d_box box;
    u32 level;
    u32 offset;
    u32 stride;
    u32 layer_stride;
};

struct drm_virtgpu_3d_transfer_from_host {
    u32 bo_handle;
    struct drm_virtgpu_3d_box box;
    u32 level;
    u32 offset;
    u32 stride;
    u32 layer_stride;
};

struct drm_virtgpu_execbuffer {
    u32 flags;
    u32 size;
    u64 command;
    u64 bo_handles;
    u32 num_bo_handles;
    s32 fence_fd;
    u32 ring_idx;
    u32 syncobj_stride;
    u32 num_in_syncobjs;
    u32 num_out_syncobjs;
    u64 in_syncobjs;
    u64 out_syncobjs;
};

struct drm_virtgpu_get_caps {
    u32 cap_set_id;
    u32 cap_set_ver;
    u64 addr;
    u32 size;
    u32 pad;
};

struct drm_virtgpu_3d_wait {
    u32 handle;
    u32 flags;
};

#define DRM_IOCTL_VIRTGPU_EXECBUFFER \
    DRM_IOWR(DRM_COMMAND_BASE + 0x02u, struct drm_virtgpu_execbuffer)
#define DRM_IOCTL_VIRTGPU_RESOURCE_CREATE \
    DRM_IOWR(DRM_COMMAND_BASE + 0x04u, struct drm_virtgpu_resource_create)
#define DRM_IOCTL_VIRTGPU_WAIT \
    DRM_IOWR(DRM_COMMAND_BASE + 0x08u, struct drm_virtgpu_3d_wait)
#define DRM_IOCTL_VIRTGPU_GET_CAPS \
    DRM_IOWR(DRM_COMMAND_BASE + 0x09u, struct drm_virtgpu_get_caps)
#define DRM_IOCTL_VIRTGPU_CONTEXT_INIT \
    DRM_IOWR(DRM_COMMAND_BASE + 0x0bu, struct drm_virtgpu_context_init)
#define DRM_IOCTL_VIRTGPU_MAP \
    DRM_IOWR(DRM_COMMAND_BASE + 0x01u, struct drm_virtgpu_map)
#define DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST \
    DRM_IOWR(DRM_COMMAND_BASE + 0x07u, struct drm_virtgpu_3d_transfer_to_host)
#define DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST \
    DRM_IOWR(DRM_COMMAND_BASE + 0x06u, struct drm_virtgpu_3d_transfer_from_host)

_Static_assert(sizeof(void *) == 8, "the demo requires AArch64 LP64");
_Static_assert(sizeof(struct drm_virtgpu_context_init) == 16, "bad context ABI");
_Static_assert(sizeof(struct drm_virtgpu_map) == 16, "bad map ABI");
_Static_assert(sizeof(struct drm_virtgpu_resource_create) == 56, "bad resource ABI");
_Static_assert(sizeof(struct drm_virtgpu_3d_transfer_to_host) == 44, "bad transfer ABI");
_Static_assert(sizeof(struct drm_virtgpu_3d_transfer_from_host) == 44, "bad readback ABI");
_Static_assert(sizeof(struct drm_virtgpu_execbuffer) == 64, "bad execbuffer ABI");
_Static_assert(sizeof(struct drm_virtgpu_get_caps) == 24, "bad caps ABI");
_Static_assert(DRM_IOCTL_VIRTGPU_CONTEXT_INIT == 0xc010644bu, "bad context ioctl");
_Static_assert(DRM_IOCTL_VIRTGPU_MAP == 0xc0106441u, "bad map ioctl");
_Static_assert(DRM_IOCTL_VIRTGPU_RESOURCE_CREATE == 0xc0386444u, "bad resource ioctl");
_Static_assert(DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST == 0xc02c6447u, "bad transfer ioctl");
_Static_assert(DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST == 0xc02c6446u, "bad readback ioctl");
_Static_assert(DRM_IOCTL_VIRTGPU_EXECBUFFER == 0xc0406442u, "bad execbuffer ioctl");

#endif
