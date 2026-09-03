#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)
#define OBJECT_BASE 64u

static const char vertex_shader[] =
    "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
static const char fragment_shader[] =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0..1]\nDCL SVIEW[0..1], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0..1]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nTEX TEMP[1], IN[0], SAMP[1], 2D\nMUL OUT[0], TEMP[0], TEMP[1]\nEND\n";

static int upload_texture(long fd, u32 bo);
static int submit_pair(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);

int virgl_run_texture_pair(long fd, const struct virgl_resources *resources)
{
    static const u8 expected[] = {55, 65, 75, 255};

    if (virgl_upload_textured_vertices(fd, resources->textured_bo, 0x3f800000u, 0x3f200000u) != 0)
        return 1;
    if (upload_texture(fd, resources->texture_pair_bo) != 0)
        return 2;
    if (submit_pair(fd, resources) != 0)
        return 3;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0)
        return 4;
    return virgl_readback_textured_triangle(fd, resources->scanout_bo, expected) == 0 ? 0 : 5;
}

static int upload_texture(long fd, u32 bo)
{
    static const u8 data[VIRGL_TEXTURE_BYTES] = {
        255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
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

static int submit_pair(long fd, const struct virgl_resources *resources)
{
    u32 words[VIRGL_TEXTURED_PAIR_WORDS] = {0};
    u32 handles[] = {
        resources->scanout_bo, resources->texture_bo, resources->texture_pair_bo, resources->textured_bo,
    };
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
    next += append_shader(words + next, OBJECT_BASE + 22u, 1, 31, fragment_shader, sizeof(fragment_shader));
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
    words[next++] = VIRGL_FORMAT_R32G32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1);
    words[next++] = OBJECT_BASE + 20u;
    words[next++] = VIRGL_HEADER(6, 0, 3);
    words[next++] = 24;
    words[next++] = 0;
    words[next++] = resources->textured_resource;
    words[next++] = VIRGL_HEADER(1, 7, 9);
    words[next++] = OBJECT_BASE + 17u;
    words[next++] = VIRGL_CLAMP_LINEAR_SAMPLER_STATE;
    next += 7;
    words[next++] = VIRGL_HEADER(1, 7, 9);
    words[next++] = OBJECT_BASE + 19u;
    words[next++] = VIRGL_REPEAT_NEAREST_SAMPLER_STATE;
    next += 7;
    words[next++] = VIRGL_HEADER(1, 6, 6);
    words[next++] = OBJECT_BASE + 18u;
    words[next++] = resources->texture_resource;
    words[next++] = VIRGL_FORMAT_R8G8B8A8_UNORM;
    next += 2;
    words[next++] = 0x688u;
    words[next++] = VIRGL_HEADER(1, 6, 6);
    words[next++] = OBJECT_BASE + 25u;
    words[next++] = resources->texture_pair_resource;
    words[next++] = VIRGL_FORMAT_R8G8B8A8_UNORM;
    next += 2;
    words[next++] = 0x688u;
    words[next++] = VIRGL_HEADER(10, 0, 4);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = OBJECT_BASE + 18u;
    words[next++] = OBJECT_BASE + 25u;
    words[next++] = VIRGL_HEADER(18, 0, 4);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = OBJECT_BASE + 17u;
    words[next++] = OBJECT_BASE + 19u;
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
