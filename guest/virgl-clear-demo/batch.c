#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define BATCH_WORDS 192u
#define OBJECT_BASE 220u
#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static const char vertex_shader[] = "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
static const char fragment_shader[] = "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";
static const u32 vertices[] = {
    0, 0x3f400000u, 0, 0x3f800000u, 0xbf400000u, 0xbf400000u, 0, 0x3f800000u,
    0x3f400000u, 0xbf400000u, 0, 0x3f800000u,
};

static int upload(long fd, u32 bo);
static int submit(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);
static int readback(long fd, u32 bo);

int virgl_run_solid_batch(long fd, const struct virgl_resources *resources)
{
    if (upload(fd, resources->triangle_bo) != 0) return 1;
    if (submit(fd, resources) != 0) return 2;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0) return 3;
    return readback(fd, resources->scanout_bo) == 0 ? 0 : 4;
}

static int upload(long fd, u32 bo)
{
    struct drm_virtgpu_3d_transfer_to_host transfer = {.bo_handle = bo, .box = {.w = sizeof(vertices), .h = 1, .d = 1}};
    u32 *mapped = (u32 *)virgl_map_buffer(fd, bo, VIRGL_TRIANGLE_BYTES);

    if (!mapped) return -1;
    for (u32 index = 0; index < sizeof(vertices) / sizeof(vertices[0]); index++) mapped[index] = vertices[index];
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &transfer) < 0 ? -2 : 0;
}

static int submit(long fd, const struct virgl_resources *resources)
{
    u32 words[BATCH_WORDS] = {0};
    u32 handles[] = {resources->scanout_bo, resources->triangle_bo};
    struct drm_virtgpu_execbuffer exec = {
        .command = (u64)words, .bo_handles = (u64)handles,
        .num_bo_handles = sizeof(handles) / sizeof(handles[0]), .fence_fd = -1,
    };

    exec.size = stream(words, resources) * sizeof(words[0]);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE;
    words[next++] = resources->scanout_resource; words[next++] = VIRGL_FORMAT_B8G8R8X8_UNORM; next += 2;
    words[next++] = VIRGL_HEADER(5, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE;
    words[next++] = VIRGL_HEADER(2, 0, 1); words[next++] = 0;
    next += append_shader(words + next, OBJECT_BASE + 1u, 0, 11, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, OBJECT_BASE + 2u, 1, 11, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 1u; words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 2u; words[next++] = 1;
    next += virgl_source_over_blend_stream(words + next, OBJECT_BASE + 3u);
    next += virgl_scissor_rasterizer_stream(words + next, OBJECT_BASE + 4u);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 5); words[next++] = OBJECT_BASE + 5u;
    words[next++] = 0; words[next++] = 0; words[next++] = 0; words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1); words[next++] = OBJECT_BASE + 5u;
    words[next++] = VIRGL_HEADER(6, 0, 3); words[next++] = 16; words[next++] = 0; words[next++] = resources->triangle_resource;
    words[next++] = VIRGL_HEADER(7, 0, 8); words[next++] = VIRGL_CLEAR_COLOR0; next += 3; words[next++] = 0x3f800000u; next += 3;
    words[next++] = VIRGL_HEADER(12, 0, 6); words[next++] = 1; words[next++] = 0;
    words[next++] = 0x3f800000u; words[next++] = 0; words[next++] = 0; words[next++] = 0x3f000000u;
    words[next++] = VIRGL_HEADER(8, 0, 12); words[next++] = 0; words[next++] = 3; words[next++] = 4; words[next++] = 0; words[next++] = 1;
    next += 5; words[next++] = ~0u; words[next++] = 0;
    words[next++] = VIRGL_HEADER(12, 0, 6); words[next++] = 1; words[next++] = 0;
    words[next++] = 0; words[next++] = 0x3f800000u; words[next++] = 0; words[next++] = 0x3f000000u;
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

static int readback(long fd, u32 bo)
{
    const u32 offset = (SCANOUT_HEIGHT / 2u * SCANOUT_WIDTH + SCANOUT_WIDTH / 2u) * 4u;
    struct drm_virtgpu_3d_transfer_from_host transfer = {
        .bo_handle = bo, .box = {.x = SCANOUT_WIDTH / 2u, .y = SCANOUT_HEIGHT / 2u, .w = 1, .h = 1, .d = 1}, .offset = offset,
    };
    u8 *pixels = virgl_map_buffer(fd, bo, SCANOUT_BYTES);

    if (!pixels || sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0) return -1;
    return pixels[offset] == 0 && pixels[offset + 1] == 128 && pixels[offset + 2] == 64 && pixels[offset + 3] == 255 ? 0 : -2;
}
