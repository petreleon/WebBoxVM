#ifndef VIRGL_CLEAR_DEMO_VIRGL_H
#define VIRGL_CLEAR_DEMO_VIRGL_H

#include "uapi.h"

#define VIRGL_TARGET_BUFFER 0u
#define VIRGL_TARGET_TEXTURE_2D 2u
#define VIRGL_FORMAT_B8G8R8X8_UNORM 2u
#define VIRGL_FORMAT_R32G32B32A32_FLOAT 31u
#define VIRGL_FORMAT_R8_UNORM 64u
#define VIRGL_BIND_RENDER_TARGET (1u << 1)
#define VIRGL_BIND_VERTEX_BUFFER (1u << 4)
#define VIRGL_CLEAR_COLOR0 (1u << 2)
#define VIRGL_CLEAR_WORDS 19u
#define VIRGL_COPY_WORDS 14u
#define VIRGL_VERTEX_INPUT_WORDS 12u
#define VIRGL_TRIANGLE_BYTES 48u
#define VIRGL_TRIANGLE_WORDS 160u
#define VIRGL_SOURCE_OVER_BLEND_WORDS 14u
#define VIRGL_SCISSOR_RASTERIZER_WORDS 12u
#define VIRGL_VIEWPORT_SCISSOR_WORDS 12u
#define VIRGL_SOURCE_OVER_BLEND_STATE \
    (1u | (3u << 4) | (19u << 9) | (1u << 17) | (19u << 22) | (15u << 27))
#define VIRGL_SCISSOR_RASTERIZER_STATE \
    ((1u << 1) | (1u << 14) | (1u << 29) | (1u << 30))
#define VIRGL_HEADER(command, object, length) \
    ((u32)(command) | ((u32)(object) << 8) | ((u32)(length) << 16))

static inline u32 virgl_source_over_blend_stream(u32 *words, u32 handle)
{
    words[0] = VIRGL_HEADER(1, 1, 11);
    words[1] = handle;
    for (u32 index = 2; index < 12; index++)
        words[index] = 0;
    words[4] = VIRGL_SOURCE_OVER_BLEND_STATE;
    words[12] = VIRGL_HEADER(2, 1, 1);
    words[13] = handle;
    return VIRGL_SOURCE_OVER_BLEND_WORDS;
}

static inline u32 virgl_scissor_rasterizer_stream(u32 *words, u32 handle)
{
    words[0] = VIRGL_HEADER(1, 2, 9);
    words[1] = handle;
    words[2] = VIRGL_SCISSOR_RASTERIZER_STATE;
    words[3] = 0x3f800000u;
    words[6] = 0x3f800000u;
    words[10] = VIRGL_HEADER(2, 2, 1);
    words[11] = handle;
    return VIRGL_SCISSOR_RASTERIZER_WORDS;
}

static inline u32 virgl_viewport_scissor_stream(u32 *words)
{
    words[0] = VIRGL_HEADER(4, 0, 7);
    words[2] = 0x43800000u;
    words[3] = 0x43400000u;
    words[4] = 0x3f000000u;
    words[5] = 0x44000000u;
    words[6] = 0x43c00000u;
    words[7] = 0x3f000000u;
    words[8] = VIRGL_HEADER(15, 0, 3);
    words[10] = 0x015001c0u;
    words[11] = 0x01b00240u;
    return VIRGL_VIEWPORT_SCISSOR_WORDS;
}

static inline void virgl_clear_stream(u32 words[VIRGL_CLEAR_WORDS], u32 resource)
{
    words[0] = VIRGL_HEADER(1, 7, 5);
    words[1] = 1;
    words[2] = resource;
    words[3] = VIRGL_FORMAT_B8G8R8X8_UNORM;
    words[4] = 0;
    words[5] = 0;
    words[6] = VIRGL_HEADER(5, 0, 3);
    words[7] = 1;
    words[8] = 0;
    words[9] = 1;
    words[10] = VIRGL_HEADER(7, 0, 8);
    words[11] = VIRGL_CLEAR_COLOR0;
    words[12] = 0x3e800000u;
    words[13] = 0x3f000000u;
    words[14] = 0x3f400000u;
    words[15] = 0x3f800000u;
    words[16] = 0;
    words[17] = 0;
    words[18] = 0;
}

static inline void virgl_copy_stream(u32 words[VIRGL_COPY_WORDS], u32 destination,
                                     u32 source)
{
    words[0] = VIRGL_HEADER(17, 0, 13);
    words[1] = destination;
    words[2] = 0;
    words[3] = 0;
    words[4] = 0;
    words[5] = 0;
    words[6] = source;
    words[7] = 0;
    words[8] = 1;
    words[9] = 0;
    words[10] = 0;
    words[11] = 2;
    words[12] = 1;
    words[13] = 1;
}

static inline void virgl_buffer_copy_stream(
    u32 words[VIRGL_COPY_WORDS], u32 destination, u32 source)
{
    words[0] = VIRGL_HEADER(17, 0, 13);
    words[1] = destination;
    words[2] = 0;
    words[3] = 4;
    words[4] = 0;
    words[5] = 0;
    words[6] = source;
    words[7] = 0;
    words[8] = 4;
    words[9] = 0;
    words[10] = 0;
    words[11] = 8;
    words[12] = 1;
    words[13] = 1;
}

static inline void virgl_vertex_input_stream(
    u32 words[VIRGL_VERTEX_INPUT_WORDS], u32 resource)
{
    words[0] = VIRGL_HEADER(1, 5, 5);
    words[1] = 9;
    words[2] = 0;
    words[3] = 0;
    words[4] = 0;
    words[5] = VIRGL_FORMAT_R8_UNORM;
    words[6] = VIRGL_HEADER(2, 5, 1);
    words[7] = 9;
    words[8] = VIRGL_HEADER(6, 0, 3);
    words[9] = 1;
    words[10] = 4;
    words[11] = resource;
}

#endif
