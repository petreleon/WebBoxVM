#include "ops.h"
#include "syscall.h"
#include "virgl.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static int virgl_caps(long fd)
{
    u8 caps[308] = {0};
    struct drm_virtgpu_get_caps get = {
        .cap_set_id = VIRTGPU_DRM_CAPSET_VIRGL,
        .cap_set_ver = 1,
        .addr = (u64)caps,
        .size = sizeof(caps),
    };

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_GET_CAPS, &get) < 0 ||
                   caps[0] != 1 || caps[4] != 2 || caps[68] != 2
               ? -1
               : 0;
}

static int init_context(long fd)
{
    struct drm_virtgpu_context_set_param parameter = {
        .param = VIRTGPU_CONTEXT_PARAM_CAPSET_ID,
        .value = VIRTGPU_DRM_CAPSET_VIRGL,
    };
    struct drm_virtgpu_context_init context = {
        .num_params = 1,
        .ctx_set_params = (u64)&parameter,
    };

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_CONTEXT_INIT, &context) < 0 ? -1 : 0;
}

static int create_resource(long fd, u32 *bo_handle, u32 *resource_handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_TEXTURE_2D,
        .format = VIRGL_FORMAT_B8G8R8A8_UNORM,
        .bind = VIRGL_BIND_RENDER_TARGET,
        .width = SCANOUT_WIDTH,
        .height = SCANOUT_HEIGHT,
        .depth = 1,
        .array_size = 1,
        .nr_samples = 1,
        .size = SCANOUT_BYTES,
        .stride = SCANOUT_WIDTH * 4u,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo_handle = resource.bo_handle;
    *resource_handle = resource.res_handle;
    return 0;
}

int virgl_setup(long fd, u32 *bo_handle, u32 *resource_handle)
{
    if (virgl_caps(fd) != 0)
        return 2;
    if (init_context(fd) != 0)
        return 3;
    return create_resource(fd, bo_handle, resource_handle) == 0 ? 0 : 4;
}

int virgl_submit_clear(long fd, u32 bo_handle, u32 resource_handle)
{
    u32 words[VIRGL_CLEAR_WORDS] = {0};
    struct drm_virtgpu_execbuffer submit = {
        .size = sizeof(words),
        .command = (u64)words,
        .bo_handles = (u64)&bo_handle,
        .num_bo_handles = 1,
        .fence_fd = -1,
    };

    virgl_clear_stream(words, resource_handle);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

int virgl_wait_for_clear(long fd, u32 bo_handle)
{
    struct drm_virtgpu_3d_wait wait = {.handle = bo_handle};

    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_WAIT, &wait) < 0 ? -1 : 0;
}
