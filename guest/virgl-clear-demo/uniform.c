#include "ops.h"
#include "syscall.h"
#include "transfer.h"
#include "virgl.h"

#define OBJECT_BASE 160u
#define UNIFORM_STORAGE_BYTES 36u
#define UNIFORM_RANGE_BYTES 16u
#define UNIFORM_OFFSET 4u
#define VERTEX_UNIFORM_OFFSET 20u
#define SCANOUT_WIDTH 1024u
#define SCANOUT_HEIGHT 768u
#define SCANOUT_BYTES (SCANOUT_WIDTH * SCANOUT_HEIGHT * 4u)

static const char vertex_shader[] =
    "VERT\nDCL IN[0]\nDCL CONST[0][0]\nDCL OUT[0], POSITION\n0: ADD OUT[0], IN[0], CONST[0][0]\n1: END\n";
static const char fragment_shader[] =
    "FRAG\nDCL CONST[0][0]\nDCL OUT[0], COLOR\nMOV OUT[0], CONST[0][0]\nEND\n";
static const u32 uniform_values[] = {
    0, 0x3e4ccccdu, 0x3f19999au, 0x3ecccccdU, 0x3f000000u,
    0xbc800000u, 0, 0, 0,
};
static int inline_write(long fd, u32 bo, u32 resource, u32 offset, const u32 *values, u32 count);
static int uniform_readback(long fd, u32 bo);
static int readback(long fd, u32 bo);
static int submit(long fd, const struct virgl_resources *resources);
static u32 stream(u32 *words, const struct virgl_resources *resources);
static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes);
int virgl_create_uniform_buffer(long fd, u32 *bo_handle, u32 *resource_handle)
{
    struct drm_virtgpu_resource_create resource = {
        .target = VIRGL_TARGET_BUFFER,
        .format = VIRGL_FORMAT_R8_UNORM,
        .bind = VIRGL_BIND_CONSTANT_BUFFER,
        .width = UNIFORM_STORAGE_BYTES,
        .height = 1,
        .depth = 1,
        .array_size = 1,
        .size = UNIFORM_STORAGE_BYTES,
    };

    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_RESOURCE_CREATE, &resource) < 0 ||
        resource.bo_handle == 0 || resource.res_handle == 0)
        return -1;
    *bo_handle = resource.bo_handle;
    *resource_handle = resource.res_handle;
    return 0;
}

int virgl_run_uniform_triangle(long fd, const struct virgl_resources *resources)
{
    if (inline_write(fd, resources->uniform_bo, resources->uniform_resource, 0, uniform_values, 5) != 0 ||
        inline_write(fd, resources->uniform_bo, resources->uniform_resource, VERTEX_UNIFORM_OFFSET, uniform_values + 5, 4) != 0)
        return 1;
    if (uniform_readback(fd, resources->uniform_bo) != 0)
        return 2;
    if (submit(fd, resources) != 0)
        return 3;
    if (virgl_wait_for_resource(fd, resources->scanout_bo) != 0)
        return 4;
    return readback(fd, resources->scanout_bo) == 0 ? 0 : 5;
}

static int inline_write(long fd, u32 bo, u32 resource, u32 offset, const u32 *values, u32 count)
{
    u32 words[VIRGL_RESOURCE_INLINE_WRITE_WORDS] = {0};
    u32 handles[] = {bo};
    struct drm_virtgpu_execbuffer exec = {
        .command = (u64)words, .bo_handles = (u64)handles, .num_bo_handles = 1, .fence_fd = -1,
    };
    if (count == 0 || count > 5)
        return -1;
    words[0] = VIRGL_HEADER(VIRGL_CCMD_RESOURCE_INLINE_WRITE, 0, 11 + count);
    words[1] = resource; words[6] = offset; words[9] = count * 4u; words[10] = 1; words[11] = 1;
    for (u32 index = 0; index < count; index++)
        words[12 + index] = values[index];
    exec.size = (12u + count) * sizeof(words[0]);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static int uniform_readback(long fd, u32 bo)
{
    struct drm_virtgpu_3d_transfer_from_host transfer = {
        .bo_handle = bo, .box = {.w = UNIFORM_STORAGE_BYTES, .h = 1, .d = 1},
    };
    u32 *mapped = (u32 *)virgl_map_buffer(fd, bo, UNIFORM_STORAGE_BYTES);

    if (!mapped)
        return -1;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0)
        return -2;
    for (u32 index = 0; index < sizeof(uniform_values) / sizeof(uniform_values[0]); index++)
        if (mapped[index] != uniform_values[index])
            return -3;
    return 0;
}

static int readback(long fd, u32 bo)
{
    static const u8 expected[] = {147, 141, 58, 255};
    const u32 y = SCANOUT_HEIGHT / 2u;
    u8 *pixels = virgl_map_buffer(fd, bo, SCANOUT_BYTES);

    if (!pixels)
        return -1;
    for (u32 x = 465; x <= 530; x += 65) {
        struct drm_virtgpu_3d_transfer_from_host transfer = {
            .bo_handle = bo, .box = {.x = x, .y = y, .w = 1, .h = 1, .d = 1},
            .offset = (y * SCANOUT_WIDTH + x) * 4u,
        };
        if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST, &transfer) < 0)
            return -2;
        if (pixels[transfer.offset] != expected[0] || pixels[transfer.offset + 1] != expected[1] ||
            pixels[transfer.offset + 2] != expected[2] || pixels[transfer.offset + 3] != expected[3])
            return -3;
    }
    return 0;
}

static int submit(long fd, const struct virgl_resources *resources)
{
    u32 words[VIRGL_TRIANGLE_WORDS] = {0};
    u32 handles[] = {
        resources->scanout_bo, resources->triangle_bo, resources->index_bo, resources->uniform_bo,
    };
    struct drm_virtgpu_execbuffer exec = {
        .command = (u64)words, .bo_handles = (u64)handles,
        .num_bo_handles = sizeof(handles) / sizeof(handles[0]), .fence_fd = -1,
    };

    exec.size = stream(words, resources) * sizeof(u32);
    return sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &exec) < 0 ? -1 : 0;
}

static u32 stream(u32 *words, const struct virgl_resources *resources)
{
    u32 next = 0;

    words[next++] = VIRGL_HEADER(1, 8, 5); words[next++] = OBJECT_BASE + 20u;
    words[next++] = resources->scanout_resource; words[next++] = VIRGL_FORMAT_B8G8R8X8_UNORM; next += 2;
    words[next++] = VIRGL_HEADER(5, 0, 3); words[next++] = 1; words[next++] = 0; words[next++] = OBJECT_BASE + 20u;
    next += append_shader(words + next, OBJECT_BASE + 21u, 0, 14, vertex_shader, sizeof(vertex_shader));
    next += append_shader(words + next, OBJECT_BASE + 22u, 1, 11, fragment_shader, sizeof(fragment_shader));
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 21u; words[next++] = 0;
    words[next++] = VIRGL_HEADER(29, 0, 2); words[next++] = OBJECT_BASE + 22u; words[next++] = 1;
    /* Command 27 consumes the offset-20 vertex vector and bytes [4, 20) color. */
    words[next++] = VIRGL_HEADER(27, 0, 5); words[next++] = 0; words[next++] = 0;
    words[next++] = VERTEX_UNIFORM_OFFSET; words[next++] = UNIFORM_RANGE_BYTES; words[next++] = resources->uniform_resource;
    words[next++] = VIRGL_HEADER(27, 0, 5); words[next++] = 1; words[next++] = 0;
    words[next++] = UNIFORM_OFFSET; words[next++] = UNIFORM_RANGE_BYTES; words[next++] = resources->uniform_resource;
    next += virgl_source_over_blend_stream(words + next, OBJECT_BASE + 23u);
    next += virgl_scissor_rasterizer_stream(words + next, OBJECT_BASE + 24u);
    next += virgl_viewport_scissor_stream(words + next);
    words[next++] = VIRGL_HEADER(1, 5, 5); words[next++] = OBJECT_BASE + 25u;
    words[next++] = 0; words[next++] = 0; words[next++] = 0;
    words[next++] = VIRGL_FORMAT_R32G32B32A32_FLOAT;
    words[next++] = VIRGL_HEADER(2, 5, 1); words[next++] = OBJECT_BASE + 25u;
    words[next++] = VIRGL_HEADER(6, 0, 3); words[next++] = 16; words[next++] = 0;
    words[next++] = resources->triangle_resource;
    words[next++] = VIRGL_HEADER(11, 0, 3); words[next++] = resources->index_resource;
    words[next++] = 2; words[next++] = 2;
    words[next++] = VIRGL_HEADER(7, 0, 8); words[next++] = VIRGL_CLEAR_COLOR0;
    words[next++] = 0x3e800000u; words[next++] = 0x3f000000u;
    words[next++] = 0x3f400000u; words[next++] = 0x3f800000u; next += 3;
    words[next++] = VIRGL_HEADER(8, 0, 12); words[next++] = 0; words[next++] = 6;
    words[next++] = 4; words[next++] = 1; words[next++] = 1; next += 5;
    words[next++] = ~0u; words[next++] = 0;
    return next;
}

static u32 append_shader(u32 *words, u32 handle, u32 kind, u32 tokens, const char *text, u32 bytes)
{
    u32 dwords = (bytes + 3u) / 4u;

    words[0] = VIRGL_HEADER(1, 4, 5 + dwords); words[1] = handle; words[2] = kind;
    words[3] = bytes; words[4] = tokens; words[5] = 0;
    for (u32 index = 0; index < bytes; index++)
        words[6 + index / 4u] |= (u32)(u8)text[index] << ((index % 4u) * 8u);
    return 6 + dwords;
}
