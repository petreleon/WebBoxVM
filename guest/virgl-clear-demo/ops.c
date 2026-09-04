#include "ops.h"
#include "syscall.h"
#include "virgl.h"

int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle)
{
    u32 words[VIRGL_CLEAR_WORDS] = {0};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)&bo_handle,
        .num_bo_handles = 1,
        .fence_fd = -1,
        .flags = VIRTGPU_EXECBUF_RING_IDX,
        .ring_idx = 1,
    };

    virgl_clear_stream(words, resource_handle);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

int virgl_submit_copy(long fd, u32 source_bo, u32 source_resource,
                      u32 destination_bo, u32 destination_resource)
{
    u32 words[VIRGL_COPY_WORDS] = {0};
    u32 handles[2] = {source_bo, destination_bo};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)handles,
        .num_bo_handles = 2,
        .fence_fd = -1,
    };

    virgl_copy_stream(words, destination_resource, source_resource);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

int virgl_submit_buffer_copy(long fd, u32 source_bo, u32 source_resource,
                             u32 destination_bo, u32 destination_resource)
{
    u32 words[VIRGL_COPY_WORDS] = {0};
    u32 handles[2] = {source_bo, destination_bo};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)handles,
        .num_bo_handles = 2,
        .fence_fd = -1,
    };

    virgl_buffer_copy_stream(words, destination_resource, source_resource);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

int virgl_wait_for_resource(long fd, u32 bo_handle)
{
    struct drm_virtgpu_3d_wait wait = {.handle = bo_handle};

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_WAIT, &wait) < 0 ? -1 : 0;
}
