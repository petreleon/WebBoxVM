#include "syscall.h"
#include "transfer.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)
#define UPLOAD_X 1u
#define UPLOAD_Y 1u
#define UPLOAD_WIDTH 2u
#define PROT_READ 1L
#define PROT_WRITE 2L
#define MAP_SHARED 1L

static u8 *mapped_pixels(long fd, u32 bo_handle)
{
    struct drm_virtgpu_map map = {.handle = bo_handle};
    u8 *pixels;

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_MAP, &map) < 0)
        return 0;
    pixels = sys_mmap(0, SCANOUT_BYTES, PROT_READ | PROT_WRITE, MAP_SHARED, fd, map.offset);
    return (long)pixels == -1 ? 0 : pixels;
}

int virgl_upload_pattern(long fd, u32 bo_handle)
{
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo_handle,
        .box = {.x = UPLOAD_X, .y = UPLOAD_Y, .w = UPLOAD_WIDTH, .h = 1, .d = 1},
        .offset = ((UPLOAD_Y * SCANOUT_WIDTH) + UPLOAD_X) * 4u,
    };
    u8 *pixels = mapped_pixels(fd, bo_handle);
    u32 offset = transfer.offset;

    if (!pixels)
        return -1;
    pixels[offset] = 10;
    pixels[offset + 1] = 20;
    pixels[offset + 2] = 30;
    pixels[offset + 3] = 255;
    pixels[offset + 4] = 40;
    pixels[offset + 5] = 50;
    pixels[offset + 6] = 60;
    pixels[offset + 7] = 255;
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}

int virgl_readback_clear(long fd, u32 bo_handle)
{
    struct drm_virtgpu_3d_transfer_from_host transfer = {
        .bo_handle = bo_handle,
        .box = {.x = UPLOAD_X, .y = UPLOAD_Y, .w = UPLOAD_WIDTH, .h = 1, .d = 1},
        .offset = ((UPLOAD_Y * SCANOUT_WIDTH) + UPLOAD_X) * 4u,
    };
    u8 *pixels = mapped_pixels(fd, bo_handle);
    u32 offset = transfer.offset;

    if (!pixels)
        return -1;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0)
        return -2;
    for (u32 index = 0; index < 8; index += 4) {
        if (pixels[offset + index] != 191 || pixels[offset + index + 1] != 128 ||
            pixels[offset + index + 2] != 64 || pixels[offset + index + 3] != 255)
            return -3;
    }
    return 0;
}
