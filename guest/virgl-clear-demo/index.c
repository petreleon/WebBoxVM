#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

int virgl_create_index_buffer(long fd, u32 *bo_handle, u32 *resource_handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_BUFFER,
        .format = VIRGL_FORMAT_R8_UNORM,
        .bind = VIRGL_BIND_INDEX_BUFFER,
        .width = VIRGL_INDEX_BUFFER_BYTES,
        .height = 1,
        .depth = 1,
        .array_size = 1,
        .size = VIRGL_INDEX_BUFFER_BYTES,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo_handle = resource.bo_handle;
    *resource_handle = resource.res_handle;
    return 0;
}

int virgl_upload_index_buffer(long fd, u32 bo_handle)
{
    static const u8 indices[VIRGL_INDEX_BUFFER_BYTES] = {
        0xa5, 0x5a, 2, 0, 1, 0, 0, 0, 5, 0, 4, 0, 3, 0,
    };
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo_handle,
        .box = {.w = VIRGL_INDEX_BUFFER_BYTES, .h = 1, .d = 1},
    };
    u8 *mapped = virgl_map_buffer(fd, bo_handle, VIRGL_INDEX_BUFFER_BYTES);

    if (!mapped)
        return -1;
    for (u32 index = 0; index < VIRGL_INDEX_BUFFER_BYTES; index++)
        mapped[index] = indices[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}
