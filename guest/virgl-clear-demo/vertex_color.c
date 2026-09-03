#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define OBJECT_BASE 96u

static const char vertex_shader[] =
    "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
static const char fragment_shader[] =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL OUT[0], COLOR[0]\nMOV OUT[0], IN[0]\nEND\n";

static int upload(long fd, u32 bo);
static int submit(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);

int virgl_create_vertex_color_resource(long fd, struct virgl_resources *resources)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_BUFFER,
        .format = VIRGL_FORMAT_R32G32B32A32_FLOAT,
        .bind = VIRGL_BIND_VERTEX_BUFFER,
        .width = VIRGL_VERTEX_COLOR_TRIANGLE_BYTES,
        .height = 1,
        .depth = 1,
        .array_size = 1,
        .size = VIRGL_VERTEX_COLOR_TRIANGLE_BYTES,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    resources->vertex_color_bo = resource.bo_handle;
    resources->vertex_color_resource = resource.res_handle;
    return 0;
}

int virgl_run_vertex_color_triangle(long fd, const struct virgl_resources *resources)
{
    static const u8 expected[] = {64, 64, 127, 255};

    if (upload(fd, resources->vertex_color_bo) != 0)
        return 1;
    if (submit(fd, resources) != 0)
        return 2;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0)
        return 3;
    return virgl_readback_scanout_pixel(fd, resources->scanout_bo, expected) == 0 ? 0 : 4;
}

static int upload(long fd, u32 bo)
{
    static const u32 vertices[] = {
        0, 0x3f400000u, 0, 0x3f800000u, 0x3f800000u, 0, 0, 0x3f800000u,
        0xbf400000u, 0xbf400000u, 0, 0x3f800000u, 0, 0x3f800000u, 0, 0x3f800000u,
        0x3f400000u, 0xbf400000u, 0, 0x3f800000u, 0, 0, 0x3f800000u, 0x3f800000u,
    };
    struct drm_virtgpu_3d_transfer_to_host transfer = {
        .bo_handle = bo,
        .box = {.w = VIRGL_VERTEX_COLOR_TRIANGLE_BYTES, .h = 1, .d = 1},
    };
    u32 *mapped = (u32 *)virgl_map_buffer(fd, bo, VIRGL_VERTEX_COLOR_TRIANGLE_BYTES);

    if (!mapped)
        return -1;
    for (u32 index = 0; index < sizeof(vertices) / sizeof(vertices[0]); index++)
        mapped[index] = vertices[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}

static int submit(long fd, const struct virgl_resources *resources)
{
    u32 words[VIRGL_VERTEX_COLOR_WORDS] = {0};
    u32 handles[] = {resources->scanout_bo, resources->vertex_color_bo};
    struct drm_virtgpu_execbuffer exec = {
        .command = (u64)words,
        .bo_handles = (u64)handles,
        .num_bo_handles = sizeof(handles) / sizeof(handles[0]),
        .fence_fd = -1,
    };

    exec.size = stream(words, resources) * sizeof(u32);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(5, 0, 3);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = 1;
    next += append_shader(words + next, OBJECT_BASE + 21u, 0, 17, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, OBJECT_BASE + 22u, 1, 11, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = OBJECT_BASE + 21u;
    words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = OBJECT_BASE + 22u;
    words[next++] = 1;
    next += virgl_source_over_blend_stream(words + next, OBJECT_BASE + 23u);
    next += virgl_scissor_rasterizer_stream(words + next, OBJECT_BASE + 24u);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 9);
    words[next++] = OBJECT_BASE + 20u;
    words[next++] = 0;
    next += 2;
    words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = 16;
    next += 2;
    words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1);
    words[next++] = OBJECT_BASE + 20u;
    words[next++] = VIRGL_HEADER(6, 0, 3);
    words[next++] = 32;
    words[next++] = 0;
    words[next++] = resources->vertex_color_resource;
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
    words[next++] = 0;
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
