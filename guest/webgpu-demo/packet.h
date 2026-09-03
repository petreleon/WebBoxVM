#ifndef WEBGPU_DEMO_PACKET_H
#define WEBGPU_DEMO_PACKET_H

#include "uapi.h"

#define WBG3_VERSION 1u
#define WBG3_DRAW_INDEXED 1u
#define WBG3_HEADER_BYTES 48u
#define WBG3_VERTEX_COUNT 8u
#define WBG3_INDEX_COUNT 36u

struct __attribute__((packed, aligned(4))) wbg3_packet {
    char magic[4];
    u32 version;
    u32 opcode;
    u32 sequence;
    u32 canvas_width;
    u32 canvas_height;
    u32 vertex_count;
    u32 index_count;
    float clear[4];
    float mvp[16];
    float vertices[WBG3_VERTEX_COUNT][7];
    u16 indices[WBG3_INDEX_COUNT];
};

extern const struct wbg3_packet cube_packet;

_Static_assert(__builtin_offsetof(struct wbg3_packet, mvp) == WBG3_HEADER_BYTES,
               "WBG3 header must be 48 bytes");
_Static_assert(__builtin_offsetof(struct wbg3_packet, vertices) == 112,
               "WBG3 MVP layout changed");
_Static_assert(__builtin_offsetof(struct wbg3_packet, indices) == 336,
               "WBG3 vertex layout changed");
_Static_assert(sizeof(struct wbg3_packet) == 408,
               "WBG3 cube packet must be exactly 408 bytes");

#endif
