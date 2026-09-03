#ifndef VIRGL_CLEAR_DEMO_VIRGL_H
#define VIRGL_CLEAR_DEMO_VIRGL_H

#include "uapi.h"

#define VIRGL_TARGET_TEXTURE_2D 2u
#define VIRGL_FORMAT_B8G8R8X8_UNORM 2u
#define VIRGL_BIND_RENDER_TARGET (1u << 1)
#define VIRGL_CLEAR_COLOR0 (1u << 2)
#define VIRGL_CLEAR_WORDS 19u
#define VIRGL_COPY_WORDS 14u
#define VIRGL_HEADER(command, object, length) \
    ((u32)(command) | ((u32)(object) << 8) | ((u32)(length) << 16))

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

#endif
