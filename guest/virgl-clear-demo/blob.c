#include "blob.h"
#include "ops.h"
#include "syscall.h"

#define BLOB_BYTES 4096u

int virgl_create_guest_blob(long fd)
{
    struct drm_virtgpu_resource_create_blob blob = {
        .blob_mem = VIRTGPU_BLOB_MEM_GUEST,
        .size = BLOB_BYTES,
    };
    struct drm_virtgpu_resource_info info;

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE_BLOB, &blob) < 0 ||
        blob.bo_handle == 0 || blob.res_handle == 0)
        return -1;
    info = (struct drm_virtgpu_resource_info){.bo_handle = blob.bo_handle};
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_INFO, &info) < 0 ||
        info.res_handle != blob.res_handle || info.size != BLOB_BYTES ||
        info.blob_mem != VIRTGPU_BLOB_MEM_GUEST)
        return -1;
    return 0;
}
