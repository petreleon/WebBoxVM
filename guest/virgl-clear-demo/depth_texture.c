#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define WORDS 256u
#define OBJECT_BASE 560u
#define VIRGL_FORMAT_Z32_FLOAT 18u
#define VIRGL_CLEAR_DEPTH 1u

static const char vertex_shader[] =
    "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
static const char fragment_shader[] =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMOV OUT[0], TEMP[0]\nEND\n";
static const u32 vertices[] = {
    0, 0x3f400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u,
    0xbf400000u, 0xbf400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u,
    0x3f400000u, 0xbf400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u,
};

static int upload(long fd, const struct virgl_resources *resources);
static int submit(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);

int virgl_run_depth_textured_triangle(long fd, const struct virgl_resources *resources)
{
    static const u8 expected[] = {10, 20, 30, 255};

    if (upload(fd, resources) != 0) return 1;
    if (submit(fd, resources) != 0) return 2;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0) return 3;
    return virgl_readback_scanout_pixel(fd, resources->scanout_bo, expected) == 0 ? 0 : 4;
}

static int upload(long fd, const struct virgl_resources *resources)
{
    static const u8 texture[] = {30, 20, 10, 255, 60, 50, 40, 255, 90, 80, 70, 255, 120, 110, 100, 255};
    struct drm_virtgpu_3d_transfer_to_host texture_transfer = {.bo_handle = resources->texture_bo, .box = {.w = 2, .h = 2, .d = 1}};
    struct drm_virtgpu_3d_transfer_to_host vertex_transfer = {.bo_handle = resources->textured_bo, .box = {.w = VIRGL_TEXTURED_TRIANGLE_BYTES, .h = 1, .d = 1}};
    u8 *texture_map = virgl_map_buffer(fd, resources->texture_bo, VIRGL_TEXTURE_BYTES);
    u32 *vertex_map = (u32 *)virgl_map_buffer(fd, resources->textured_bo, VIRGL_TEXTURED_TRIANGLE_BYTES);

    if (!texture_map || !vertex_map) return -1;
    for (u32 index = 0; index < sizeof(texture); index++) texture_map[index] = texture[index];
    for (u32 index = 0; index < sizeof(vertices) / sizeof(vertices[0]); index++) vertex_map[index] = vertices[index];
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &texture_transfer) < 0) return -2;
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &vertex_transfer) < 0 ? -3 : 0;
}

static int submit(long fd, const struct virgl_resources *resources)
{
    u32 words[WORDS] = {0};
    u32 handles[] = {resources->scanout_bo, resources->depth_bo, resources->texture_bo, resources->textured_bo};
    struct drm_virtgpu_execbuffer exec = {.command = (u64)words, .bo_handles = (u64)handles,
        .num_bo_handles = sizeof(handles) / sizeof(handles[0]), .fence_fd = -1};

    exec.size = stream(words, resources) * sizeof(words[0]);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE;
    words[next++] = resources->scanout_resource; words[next++] = VIRGL_FORMAT_B8G8R8X8_UNORM; next += 2;
    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE + 1u;
    words[next++] = resources->depth_resource; words[next++] = VIRGL_FORMAT_Z32_FLOAT; next += 2;
    words[next++] = VIRGL_HEADER(5, 0, 3); words[next++] = 1; words[next++] = OBJECT_BASE + 1u; words[next++] = OBJECT_BASE;
    next += append_shader(words + next, OBJECT_BASE + 2u, 0, 17, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, OBJECT_BASE + 3u, 1, 25, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 2u; words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 3u; words[next++] = 1;
    next += virgl_source_over_blend_stream(words + next, OBJECT_BASE + 4u);
    next += virgl_scissor_rasterizer_stream(words + next, OBJECT_BASE + 5u);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 9); words[next++] = OBJECT_BASE + 6u;
    words[next++] = 0; next += 2; words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = 16; next += 2; words[next++] = VIRGL_FORMAT_R32G32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1); words[next++] = OBJECT_BASE + 6u;
    words[next++] = VIRGL_HEADER(6, 0, 3); words[next++] = 24; words[next++] = 0; words[next++] = resources->textured_resource;
    words[next++] = VIRGL_HEADER(1, 7, 9); words[next++] = OBJECT_BASE + 7u; words[next++] = VIRGL_CLAMP_NEAREST_SAMPLER_STATE; next += 7;
    words[next++] = VIRGL_HEADER(1, 6, 6); words[next++] = OBJECT_BASE + 8u; words[next++] = resources->texture_resource;
    words[next++] = VIRGL_FORMAT_R8G8B8A8_UNORM; next += 2; words[next++] = 0x688u;
    words[next++] = VIRGL_HEADER(10, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE + 8u;
    words[next++] = VIRGL_HEADER(18, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE + 7u;
    words[next++] = VIRGL_HEADER(1, 0, 5); words[next++] = OBJECT_BASE + 9u; words[next++] = 7; next += 3;
    words[next++] = VIRGL_HEADER(2, 0, 1); words[next++] = OBJECT_BASE + 9u;
    words[next++] = VIRGL_HEADER(7, 0, 8); words[next++] = VIRGL_CLEAR_COLOR0 | VIRGL_CLEAR_DEPTH;
    words[next++] = 0x3e800000u; words[next++] = 0x3f000000u; words[next++] = 0x3f400000u; words[next++] = 0x3f800000u;
    words[next++] = 0x3f800000u; next += 2;
    words[next++] = VIRGL_HEADER(8, 0, 12); words[next++] = 0; words[next++] = 3; words[next++] = 4; words[next++] = 0; words[next++] = 1;
    next += 5; words[next++] = ~0u; words[next++] = 0;
    return next;
}

static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes)
{
    u32 dwords = (bytes + 3u) / 4u;

    words[0] = VIRGL_HEADER(1, 4, 5 + dwords); words[1] = handle; words[2] = kind;
    words[3] = bytes; words[4] = tokens; words[5] = 0;
    for (u32 index = 0; index < bytes; index++) words[6 + index / 4u] |= (u32)(u8)text[index] << ((index % 4u) * 8u);
    return 6 + dwords;
}
