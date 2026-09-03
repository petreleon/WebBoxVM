#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

static int create_buffer(long fd, u32 *bo, u32 *resource);
static int create_texture(long fd, u32 *bo, u32 *resource);
static int upload_vertices(long fd, u32 bo);
static int upload_texture(long fd, u32 bo);

int virgl_create_textured_resources(long fd, struct virgl_resources *resources)
{
    if (create_texture(fd, &resources->texture_bo, &resources->texture_resource) != 0)
        return -1;
    return create_buffer(fd, &resources->textured_bo, &resources->textured_resource);
}

int virgl_run_textured_triangle(long fd, const struct virgl_resources *resources)
{
    if (upload_texture(fd, resources->texture_bo) != 0)
        return 1;
    if (upload_vertices(fd, resources->textured_bo) != 0)
        return 2;
    if (virgl_submit_textured_triangle(fd, resources) != 0)
        return 3;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0)
        return 4;
    return virgl_readback_textured_triangle(fd, resources->scanout_bo) == 0 ? 0 : 5;
}

static int create_buffer(long fd, u32 *bo, u32 *handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_BUFFER,
        .format = VIRGL_FORMAT_R32G32B32A32_FLOAT,
        .bind = VIRGL_BIND_VERTEX_BUFFER,
        .width = VIRGL_TEXTURED_TRIANGLE_BYTES,
        .height = 1,
        .depth = 1,
        .array_size = 1,
        .size = VIRGL_TEXTURED_TRIANGLE_BYTES,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo = resource.bo_handle;
    *handle = resource.res_handle;
    return 0;
}

static int create_texture(long fd, u32 *bo, u32 *handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_TEXTURE_2D,
        .format = VIRGL_FORMAT_R8G8B8A8_UNORM,
        .bind = VIRGL_BIND_SAMPLER_VIEW,
        .width = 2,
        .height = 2,
        .depth = 1,
        .array_size = 1,
        .size = VIRGL_TEXTURE_BYTES,
        .stride = 8,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo = resource.bo_handle;
    *handle = resource.res_handle;
    return 0;
}

static int upload_vertices(long fd, u32 bo)
{
    static const u32 data[] = {
        0, 0x3f400000u, 0, 0x3f800000u, 0x3f800000u, 0x3f800000u,
        0xbf400000u, 0xbf400000u, 0, 0x3f800000u, 0x3f800000u, 0x3f800000u,
        0x3f400000u, 0xbf400000u, 0, 0x3f800000u, 0x3f800000u, 0x3f800000u,
    };
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo,
        .box = {.w = VIRGL_TEXTURED_TRIANGLE_BYTES, .h = 1, .d = 1},
    };
    u32 *mapped = (u32 *)virgl_map_buffer(fd, bo, VIRGL_TEXTURED_TRIANGLE_BYTES);

    if (!mapped)
        return -1;
    for (u32 index = 0; index < sizeof(data) / sizeof(data[0]); index++)
        mapped[index] = data[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}

static int upload_texture(long fd, u32 bo)
{
    static const u8 data[VIRGL_TEXTURE_BYTES] = {
        30, 20, 10, 255, 60, 50, 40, 255, 90, 80, 70, 255, 120, 110, 100, 255,
    };
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo,
        .box = {.w = 2, .h = 2, .d = 1},
    };
    u8 *mapped = virgl_map_buffer(fd, bo, VIRGL_TEXTURE_BYTES);

    if (!mapped)
        return -1;
    for (u32 index = 0; index < VIRGL_TEXTURE_BYTES; index++)
        mapped[index] = data[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}
