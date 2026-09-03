#ifndef WEBGPU_DEMO_UAPI_H
#define WEBGPU_DEMO_UAPI_H

typedef unsigned short u16;
typedef unsigned int u32;
typedef signed int s32;
typedef unsigned long u64;

#define WBG3_CAPSET_ID 7u
#define VIRTGPU_CONTEXT_PARAM_CAPSET_ID 1u

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

struct drm_virtgpu_context_set_param {
    u64 param;
    u64 value;
};

struct drm_virtgpu_context_init {
    u32 num_params;
    u32 pad;
    u64 ctx_set_params;
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

#define DRM_IOCTL_VIRTGPU_EXECBUFFER \
    DRM_IOWR(DRM_COMMAND_BASE + 0x02u, struct drm_virtgpu_execbuffer)
#define DRM_IOCTL_VIRTGPU_CONTEXT_INIT \
    DRM_IOWR(DRM_COMMAND_BASE + 0x0bu, struct drm_virtgpu_context_init)

_Static_assert(sizeof(void *) == 8, "the demo requires AArch64 LP64");
_Static_assert(sizeof(struct drm_virtgpu_context_set_param) == 16,
               "unexpected context parameter ABI");
_Static_assert(sizeof(struct drm_virtgpu_context_init) == 16,
               "unexpected context init ABI");
_Static_assert(sizeof(struct drm_virtgpu_execbuffer) == 64,
               "unexpected execbuffer ABI");
_Static_assert(DRM_IOCTL_VIRTGPU_CONTEXT_INIT == 0xc010644bu,
               "unexpected CONTEXT_INIT ioctl number");
_Static_assert(DRM_IOCTL_VIRTGPU_EXECBUFFER == 0xc0406442u,
               "unexpected EXECBUFFER ioctl number");

#endif
