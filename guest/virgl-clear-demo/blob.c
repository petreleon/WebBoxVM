#include "blob.h"
#include "ops.h"
#include "syscall.h"

#define BLOB_BYTES 4096u
#define PROT_READ 1L
#define PROT_WRITE 2L
#define MAP_SHARED 1L

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

int virgl_create_host_blob(long fd)
{
    struct drm_virtgpu_resource_create_blob blob = {
        .blob_mem = VIRTGPU_BLOB_MEM_HOST3D,
        .blob_flags = VIRTGPU_BLOB_FLAG_USE_MAPPABLE,
        .size = BLOB_BYTES,
    };
    struct drm_virtgpu_resource_info info;
    struct drm_virtgpu_map map;
    volatile u32 *words;

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE_BLOB, &blob) < 0 ||
        blob.bo_handle == 0 || blob.res_handle == 0)
        return -1;
    info = (struct drm_virtgpu_resource_info){.bo_handle = blob.bo_handle};
    map = (struct drm_virtgpu_map){.handle = blob.bo_handle};
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_INFO, &info) < 0 ||
        info.res_handle != blob.res_handle || info.size != BLOB_BYTES ||
        info.blob_mem != VIRTGPU_BLOB_MEM_HOST3D ||
        sys_ioctl(fd, DRM_IOCTL_VIRTGPU_MAP, &map) < 0)
        return -1;
    words = sys_mmap(0, BLOB_BYTES, PROT_READ | PROT_WRITE, MAP_SHARED, fd, map.offset);
    if (words == (void *)-1)
        return -1;
    words[0] = 0x56454e55u;
    return words[0] == 0x56454e55u ? 0 : -1;
}
