#include "ops.h"
#include "syscall.h"
#include "virgl.h"

int virgl_submit_vertex_input(long fd, u32 bo_handle, u32 resource_handle)
{
    u32 words[VIRGL_VERTEX_INPUT_WORDS] = {0};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)&bo_handle,
        .num_bo_handles = 1,
        .fence_fd = -1,
    };

    virgl_vertex_input_stream(words, resource_handle);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}
