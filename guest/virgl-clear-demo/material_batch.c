#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define WORDS 448u
#define OBJECT_BASE 640u
#define VIRGL_FORMAT_Z32_FLOAT 18u
#define VIRGL_CLEAR_DEPTH 1u

static const char solid_vert[] = "VERT\nDCL IN[0]\nDCL OUT[0], POSITION\n0: MOV OUT[0], IN[0]\n1: END\n";
static const char solid_frag[] = "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";
static const char texture_vert[] = "VERT\nDCL IN[0..1]\nDCL OUT[0], POSITION\nDCL OUT[1], GENERIC[0]\nMOV OUT[0], IN[0]\nMOV OUT[1], IN[1]\nEND\n";
static const char texture_frag[] = "FRAG\nDCL CONST[0][0]\nDCL IN[0], GENERIC[0], LINEAR\nDCL SAMP[0]\nDCL SVIEW[0], 2D, FLOAT\nDCL OUT[0], COLOR[0]\nDCL TEMP[0]\nTEX TEMP[0], IN[0], SAMP[0], 2D\nMUL OUT[0], TEMP[0], CONST[0][0]\nEND\n";
static const u32 solid_vertices[] = {0, 0x3f400000u, 0x3f000000u, 0x3f800000u, 0xbf400000u, 0xbf400000u, 0x3f000000u, 0x3f800000u, 0x3f400000u, 0xbf400000u, 0x3f000000u, 0x3f800000u};
static const u32 texture_vertices[] = {0, 0x3f400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u, 0xbf400000u, 0xbf400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u, 0x3f400000u, 0xbf400000u, 0xbf000000u, 0x3f800000u, 0, 0x3f800000u};

static int upload(long fd, const struct virgl_resources *resources); static int submit(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources); static u32 shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);

int virgl_run_material_batch(long fd, const struct virgl_resources *resources)
{
    static const u8 expected[] = {32, 32, 64, 255};
    if (upload(fd, resources) != 0) return 1;
    if (submit(fd, resources) != 0) return 2;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0) return 3;
    return virgl_readback_scanout_pixel(fd, resources->scanout_bo, expected) == 0 ? 0 : 4;
}

static int upload(long fd, const struct virgl_resources *resources)
{
    static const u8 texels[] = {128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255};
    struct drm_virtgpu_3d_transfer_to_host solid = {.bo_handle = resources->depth_vertex_bo, .box = {.w = sizeof(solid_vertices), .h = 1, .d = 1}};
    struct drm_virtgpu_3d_transfer_to_host textured = {.bo_handle = resources->textured_bo, .box = {.w = sizeof(texture_vertices), .h = 1, .d = 1}};
    struct drm_virtgpu_3d_transfer_to_host texture = {.bo_handle = resources->texture_bo, .box = {.w = 2, .h = 2, .d = 1}};
    u32 *solid_map = (u32 *)virgl_map_buffer(fd, resources->depth_vertex_bo, sizeof(solid_vertices));
    u32 *textured_map = (u32 *)virgl_map_buffer(fd, resources->textured_bo, sizeof(texture_vertices));
    u8 *texture_map = virgl_map_buffer(fd, resources->texture_bo, sizeof(texels));
    if (!solid_map || !textured_map || !texture_map) return -1;
    for (u32 i = 0; i < sizeof(solid_vertices) / sizeof(*solid_vertices); i++) solid_map[i] = solid_vertices[i];
    for (u32 i = 0; i < sizeof(texture_vertices) / sizeof(*texture_vertices); i++) textured_map[i] = texture_vertices[i];
    for (u32 i = 0; i < sizeof(texels); i++) texture_map[i] = texels[i];
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &solid) < 0) return -2;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &textured) < 0) return -3;
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST, &texture) < 0 ? -4 : 0;
}

static int submit(long fd, const struct virgl_resources *resources)
{
    u32 words[WORDS] = {0}; u32 handles[] = {resources->scanout_bo, resources->depth_bo, resources->depth_vertex_bo, resources->texture_bo, resources->textured_bo};
    struct drm_virtgpu_execbuffer exec = {.command = (u64)words, .bo_handles = (u64)handles, .num_bo_handles = sizeof(handles) / sizeof(*handles), .fence_fd = -1};
    exec.size = stream(words, resources) * sizeof(*words);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;
    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE; words[next++] = resources->scanout_resource; words[next++] = VIRGL_FORMAT_B8G8R8X8_UNORM; next += 2;
    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE + 1u; words[next++] = resources->depth_resource; words[next++] = VIRGL_FORMAT_Z32_FLOAT; next += 2;
    words[next++] = VIRGL_HEADER(5, 0, 3); words[next++] = 1; words[next++] = OBJECT_BASE + 1u; words[next++] = OBJECT_BASE;
    next += shader(words + next, OBJECT_BASE + 2u, 0, 11, solid_vert, sizeof(solid_vert)); next += shader(words + next, OBJECT_BASE + 3u, 1, 11, solid_frag, sizeof(solid_frag));
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 2u; words[next++] = 0; words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 3u; words[next++] = 1;
    words[next++] = VIRGL_HEADER(12, 0, 6); words[next++] = 1; words[next++] = 0; words[next++] = 0x3f800000u; words[next++] = 0; words[next++] = 0; words[next++] = 0x3f000000u;
    next += virgl_source_over_blend_stream(words + next, OBJECT_BASE + 4u); next += virgl_scissor_rasterizer_stream(words + next, OBJECT_BASE + 5u); next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 5); words[next++] = OBJECT_BASE + 6u; words[next++] = 0; words[next++] = 0; words[next++] = 0; words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1); words[next++] = OBJECT_BASE + 6u; words[next++] = VIRGL_HEADER(6, 0, 3); words[next++] = 16; words[next++] = 0; words[next++] = resources->depth_vertex_resource;
    words[next++] = VIRGL_HEADER(1, 0, 5); words[next++] = OBJECT_BASE + 7u; words[next++] = 7; next += 3; words[next++] = VIRGL_HEADER(2, 0, 1); words[next++] = OBJECT_BASE + 7u;
    words[next++] = VIRGL_HEADER(7, 0, 8); words[next++] = VIRGL_CLEAR_COLOR0 | VIRGL_CLEAR_DEPTH; words[next++] = 0x3e800000u; words[next++] = 0x3f000000u; words[next++] = 0x3f400000u; words[next++] = 0x3f800000u; words[next++] = 0x3f800000u; next += 2;
    words[next++] = VIRGL_HEADER(8, 0, 12); words[next++] = 0; words[next++] = 3; words[next++] = 4; words[next++] = 0; words[next++] = 1; next += 5; words[next++] = ~0u; words[next++] = 0;
    next += shader(words + next, OBJECT_BASE + 8u, 0, 17, texture_vert, sizeof(texture_vert)); next += shader(words + next, OBJECT_BASE + 9u, 1, 30, texture_frag, sizeof(texture_frag));
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 8u; words[next++] = 0; words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 9u; words[next++] = 1;
    words[next++] = VIRGL_HEADER(1, 5, 9); words[next++] = OBJECT_BASE + 10u; words[next++] = 0; next += 2; words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT; words[next++] = 16; next += 2; words[next++] = VIRGL_FORMAT_R32G32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1); words[next++] = OBJECT_BASE + 10u; words[next++] = VIRGL_HEADER(6, 0, 3); words[next++] = 24; words[next++] = 0; words[next++] = resources->textured_resource;
    words[next++] = VIRGL_HEADER(1, 7, 9); words[next++] = OBJECT_BASE + 11u; words[next++] = VIRGL_CLAMP_NEAREST_SAMPLER_STATE; next += 7;
    words[next++] = VIRGL_HEADER(1, 6, 6); words[next++] = OBJECT_BASE + 12u; words[next++] = resources->texture_resource; words[next++] = VIRGL_FORMAT_R8G8B8A8_UNORM; next += 2; words[next++] = 0x688u;
    words[next++] = VIRGL_HEADER(10, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE + 12u; words[next++] = VIRGL_HEADER(18, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE + 11u;
    words[next++] = VIRGL_HEADER(12, 0, 6); words[next++] = 1; words[next++] = 0; words[next++] = 0x3f000000u; words[next++] = 0x3f000000u; words[next++] = 0x3f000000u; words[next++] = 0x3f800000u;
    words[next++] = VIRGL_HEADER(8, 0, 12); words[next++] = 0; words[next++] = 3; words[next++] = 4; words[next++] = 0; words[next++] = 1; next += 5; words[next++] = ~0u; words[next++] = 0;
    return next;
}

static u32 shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes)
{
    u32 dwords = (bytes + 3u) / 4u; words[0] = VIRGL_HEADER(1, 4, 5 + dwords); words[1] = handle; words[2] = kind; words[3] = bytes; words[4] = tokens; words[5] = 0;
    for (u32 i = 0; i < bytes; i++) words[6 + i / 4u] |= (u32)(u8)text[i] << ((i % 4u) * 8u);
    return 6 + dwords;
}
