#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static const char vertex_shader[] =
    "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
static const char fragment_shader[] =
    "FRAG\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMOV OUT[0], TEMP[0]\nEND\n";

static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);

int virgl_submit_textured_triangle(long fd, const struct virgl_resources *resources)
{
    u32 words[VIRGL_TEXTURED_TRIANGLE_WORDS] = {0};
    u32 handles[3] = {resources->scanout_bo, resources->texture_bo, resources->textured_bo};
    struct drm_virtgpu_execbuffer exec = {
        .command = (u64)words,
        .bo_handles = (u64)handles,
        .num_bo_handles = 3,
        .fence_fd = -1,
    };

    exec.size = stream(words, resources) * sizeof(u32);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

int virgl_readback_textured_triangle(long fd, u32 bo)
{
    const u32 x = SCANOUT_WIDTH / 2u;
    const u32 y = SCANOUT_HEIGHT / 2u;
    struct drm_virtgpu_3d_transfer_from_host transfer = {
        .bo_handle = bo,
        .box = {.x = x, .y = y, .w = 1, .h = 1, .d = 1},
        .offset = (y * SCANOUT_WIDTH + x) * 4u,
    };
    u8 *pixels = virgl_map_buffer(fd, bo, SCANOUT_BYTES);

    if (!pixels)
        return -1;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0)
        return -2;
    return pixels[transfer.offset] == 10 && pixels[transfer.offset + 1] == 20 &&
                   pixels[transfer.offset + 2] == 30 && pixels[transfer.offset + 3] == 255
               ? 0 : -3;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(5, 0, 3);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = 1;
    next += append_shader(words + next, 21, 0, 17, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, 22, 1, 25, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = 21;
    words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2);
    words[next++] = 22;
    words[next++] = 1;
    next += virgl_source_over_blend_stream(words + next, 23);
    next += virgl_scissor_rasterizer_stream(words + next, 24);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 9);
    words[next++] = 20;
    words[next++] = 0;
    next += 2;
    words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = 16;
    next += 2;
    words[next++] = VIRGL_FORMAT_R32G32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1);
    words[next++] = 20;
    words[next++] = VIRGL_HEADER(6, 0, 3);
    words[next++] = 24;
    words[next++] = 0;
    words[next++] = resources->textured_resource;
    words[next++] = VIRGL_HEADER(1, 7, 9);
    words[next++] = 17;
    words[next++] = 0x1092u;
    next += 7;
    words[next++] = VIRGL_HEADER(1, 6, 6);
    words[next++] = 18;
    words[next++] = resources->texture_resource;
    words[next++] = VIRGL_FORMAT_B8G8R8A8_UNORM;
    words[next++] = 0;
    words[next++] = 0;
    words[next++] = 0x688u;
    words[next++] = VIRGL_HEADER(10, 0, 3);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = 18;
    words[next++] = VIRGL_HEADER(18, 0, 3);
    words[next++] = 1;
    words[next++] = 0;
    words[next++] = 17;
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
