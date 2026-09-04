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

static int virgl_create_default_blob(long fd)
{
    struct drm_virtgpu_resource_create_blob blob = {
        .blob_mem = VIRTGPU_BLOB_MEM_HOST3D_GUEST,
        .size = BLOB_BYTES,
    };
    struct drm_virtgpu_resource_info info;
    struct drm_virtgpu_map map;
    struct drm_virtgpu_3d_transfer_to_host upload;
    struct drm_virtgpu_3d_transfer_from_host download;
    volatile u32 *words;

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE_BLOB, &blob) < 0 ||
        blob.bo_handle == 0 || blob.res_handle == 0)
        return -1;
    info = (struct drm_virtgpu_resource_info){.bo_handle = blob.bo_handle};
    map = (struct drm_virtgpu_map){.handle = blob.bo_handle};
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_INFO, &info) < 0 ||
        info.res_handle != blob.res_handle || info.size != BLOB_BYTES ||
        info.blob_mem != VIRTGPU_BLOB_MEM_HOST3D_GUEST ||
        sys_ioctl(fd, DRM_IOCTL_VIRTGPU_MAP, &map) < 0)
        return -1;
    words = sys_mmap(0, BLOB_BYTES, PROT_READ | PROT_WRITE, MAP_SHARED, fd, map.offset);
    if (words == (void *)-1)
        return -1;
    upload = (struct drm_virtgpu_3d_transfer_to_host){
        .bo_handle = blob.bo_handle, .box = {.w = sizeof(*words), .h = 1, .d = 1},
    };
    download = (struct drm_virtgpu_3d_transfer_from_host){
        .bo_handle = blob.bo_handle, .box = {.w = sizeof(*words), .h = 1, .d = 1},
    };
    words[0] = 0x53484457u;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &upload) < 0)
        return -1;
    words[0] = 0;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &download) < 0)
        return -1;
    return words[0] == 0x53484457u ? 0 : -1;
}

int virgl_verify_blob_profiles(long fd)
{
    return virgl_create_guest_blob(fd) || virgl_create_host_blob(fd) ||
                   virgl_create_default_blob(fd)
               ? -1
               : 0;
}
