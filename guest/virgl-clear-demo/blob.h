#ifndef VIRGL_CLEAR_DEMO_BLOB_H
#define VIRGL_CLEAR_DEMO_BLOB_H

#include "uapi.h"

#define VIRTGPU_BLOB_MEM_GUEST 0x0001u
#define VIRTGPU_BLOB_MEM_HOST3D 0x0002u
#define VIRTGPU_BLOB_MEM_HOST3D_GUEST 0x0003u
#define VIRTGPU_BLOB_FLAG_USE_MAPPABLE 0x0001u
#define VIRTGPU_RESOURCE_CREATE_BLOB 0x0au
#define VIRTGPU_RESOURCE_INFO 0x05u

struct drm_virtgpu_resource_info {
    u32 bo_handle;
    u32 res_handle;
    u32 size;
    u32 blob_mem;
};

struct drm_virtgpu_resource_create_blob {
    u32 blob_mem;
    u32 blob_flags;
    u32 bo_handle;
    u32 res_handle;
    u64 size;
    u32 pad;
    u32 cmd_size;
    u64 cmd;
    u64 blob_id;
};

struct virgl_renderer_blob_prepare {
    u8 magic[4];
    u32 version;
    u64 blob_id;
    u64 size;
    u32 blob_mem;
    u32 blob_flags;
};

#define DRM_IOCTL_VIRTGPU_RESOURCE_INFO \
    DRM_IOWR(DRM_COMMAND_BASE + VIRTGPU_RESOURCE_INFO, struct drm_virtgpu_resource_info)
#define DRM_IOCTL_VIRTGPU_RESOURCE_CREATE_BLOB \
    DRM_IOWR(DRM_COMMAND_BASE + VIRTGPU_RESOURCE_CREATE_BLOB, struct drm_virtgpu_resource_create_blob)

_Static_assert(sizeof(struct drm_virtgpu_resource_info) == 16, "bad blob-info ABI");
_Static_assert(sizeof(struct drm_virtgpu_resource_create_blob) == 48, "bad blob-create ABI");
_Static_assert(sizeof(struct virgl_renderer_blob_prepare) == 32, "bad blob-prepare ABI");
_Static_assert(DRM_IOCTL_VIRTGPU_RESOURCE_CREATE_BLOB == 0xc030644au, "bad blob-create ioctl");

#endif
