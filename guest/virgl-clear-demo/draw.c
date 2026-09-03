#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static const char vertex_shader[] =
    "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
static const char fragment_shader[] =
    "FRAG\nDCL OUT[0], COLOR\nIMM[0] FLT32 {0, 1, 0, .25}\n0: MOV OUT[0], IMM[0]\n1: END\n";

static int submit_triangle(long fd, u32 scanout_bo, u32 triangle_bo, u32 triangle_resource, u32 index_bo, u32 index_resource);
static u32 triangle_stream(u32 *words, u32 triangle, u32 index);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);
static int upload_triangle(long fd, u32 bo_handle);
static int readback_triangle(long fd, u32 bo_handle);

int virgl_create_triangle_buffer(long fd, u32 *bo_handle, u32 *resource_handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_BUFFER,
        .format = VIRGL_FORMAT_R32G32B32A32_FLOAT,
        .bind = VIRGL_BIND_VERTEX_BUFFER,
        .width = VIRGL_TRIANGLE_BYTES,
        .height = 1,
        .depth = 1,
        .array_size = 1,
        .size = VIRGL_TRIANGLE_BYTES,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo_handle = resource.bo_handle;
    *resource_handle = resource.res_handle;
    return 0;
}

int virgl_run_triangle(long fd, const struct virgl_resources *resources)
{
    if (upload_triangle(fd, resources->triangle_bo) != 0)
        return 1;
    if (virgl_upload_index_buffer(fd, resources->index_bo) != 0)
        return 1;
    if (submit_triangle(fd, resources->scanout_bo, resources->triangle_bo,
                        resources->triangle_resource, resources->index_bo,
                        resources->index_resource) != 0)
        return 2;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0)
        return 3;
    return readback_triangle(fd, resources->scanout_bo) == 0 ? 0 : 4;
}

static int submit_triangle(long fd, u32 scanout_bo, u32 triangle_bo, u32 triangle_resource, u32 index_bo, u32 index_resource)
{
    u32 words[VIRGL_TRIANGLE_WORDS] = {0};
    u32 handles[3] = {scanout_bo, triangle_bo, index_bo};
    struct drm_virtgpu_execbuffer submit = {
        .command = (u64)words,
        .bo_handles = (u64)handles,
        .num_bo_handles = 3,
        .fence_fd = -1,
    };

    submit.size = triangle_stream(words, triangle_resource, index_resource) * sizeof(u32);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0 ? -1 : 0;
}

static u32 triangle_stream(u32 *words, u32 triangle, u32 index)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(5, 0, 3);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = 1;
    next += append_shader(words + next, 11, 0, 11, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, 12, 1, 14, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = 11;
    words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = 12;
    words[next++] = 1;
    next += virgl_source_over_blend_stream(words + next, 13);
    next += virgl_scissor_rasterizer_stream(words + next, 14);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 5);
    words[next++] = 10;
    words[next++] = 0;
    words[next++] = 0;
    words[next++] = 0;
    words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1);
    words[next++] = 10;
    words[next++] = VIRGL_HEADER(6, 0, 3);
    words[next++] = 16;
    words[next++] = 0;
    words[next++] = triangle;
    words[next++] = VIRGL_HEADER(11, 0, 3);
    words[next++] = index;
    words[next++] = 2;
    words[next++] = 0;
    words[next++] = VIRGL_HEADER(7, 0, 8);
    words[next++] = VIRGL_CLEAR_COLOR0;
    words[next++] = 0x3e800000u;
    words[next++] = 0x3f000000u;
    words[next++] = 0x3f400000u;
    words[next++] = 0x3f800000u;
    next += 3;
    words[next++] = VIRGL_HEADER(8, 0, 12);
    words[next++] = 0;
    words[next++] = 3;
    words[next++] = 4;
    words[next++] = 1;
    words[next++] = 1;
    next += 5;
    words[next++] = ~0u;
    words[next++] = 0;
    return next;
}

static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes)
{
    u32 dwords = (bytes + 3u) / 4u;

    words[0] = VIRGL_HEADER(1, 4, 5 + dwords);
    words[1] = handle;
    words[2] = kind;
    words[3] = bytes;
    words[4] = tokens;
    words[5] = 0;
    for (u32 index = 0; index < bytes; index++)
        words[6 + index / 4u] |= (u32)(u8)text[index] << ((index % 4u) * 8u);
    return 6 + dwords;
}

static int upload_triangle(long fd, u32 bo_handle)
{
    static const u32 positions[] = {
        0, 0x3f400000u, 0, 0x3f800000u,
        0xbf400000u, 0xbf400000u, 0, 0x3f800000u,
        0x3f400000u, 0xbf400000u, 0, 0x3f800000u,
    };
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo_handle,
        .box = {.w = VIRGL_TRIANGLE_BYTES, .h = 1, .d = 1},
    };
    u32 *pixels = (u32 *)virgl_map_buffer(fd, bo_handle, VIRGL_TRIANGLE_BYTES);

    if (!pixels)
        return -1;
    for (u32 index = 0; index < sizeof(positions) / sizeof(positions[0]); index++)
        pixels[index] = positions[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}

static int readback_triangle(long fd, u32 bo_handle)
{
    const u32 x = SCANOUT_WIDTH / 2u;
    const u32 y = SCANOUT_HEIGHT / 2u;
    struct drm_virtgpu_3d_transfer_from_host transfer = {
        .bo_handle = bo_handle,
        .box = {.x = x, .y = y, .w = 1, .h = 1, .d = 1},
        .offset = (y * SCANOUT_WIDTH + x) * 4u,
    };
    u8 *pixels = virgl_map_buffer(fd, bo_handle, SCANOUT_BYTES);
    u32 offset = transfer.offset;

    if (!pixels)
        return -1;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0)
        return -2;
    return pixels[offset] == 143 && pixels[offset + 1] == 160 &&
                   pixels[offset + 2] == 48 && pixels[offset + 3] == 255
               ? 0 : -3;
}
