#include "packet.h"

/* Column-major fixed orthographic rotation with WebGPU depth in [0, 1]. */
const struct wbg3_packet cube_packet
    __attribute__((used, section(".rodata.wbg3"))) = {
        .magic = {'W', 'B', 'G', '3'},
        .version = WBG3_VERSION,
        .opcode = WBG3_DRAW_INDEXED,
        .sequence = 1,
        .canvas_width = 1024,
        .canvas_height = 768,
        .vertex_count = WBG3_VERTEX_COUNT,
        .index_count = WBG3_INDEX_COUNT,
        .clear = {0.015625f, 0.0234375f, 0.046875f, 1.0f},
        .mvp = {
            0.573406f, 0.0f, -0.258109f, 0.0f,
            -0.169683f, 0.634416f, -0.155785f, 0.0f,
            0.363886f, 0.295833f, 0.334082f, 0.0f,
            0.0f, 0.0f, 0.5f, 1.0f,
        },
        .vertices = {
            {-0.5f, -0.5f, -0.5f, 1.0f, 0.1f, 0.1f, 1.0f},
            { 0.5f, -0.5f, -0.5f, 0.1f, 1.0f, 0.1f, 1.0f},
            { 0.5f,  0.5f, -0.5f, 0.1f, 0.3f, 1.0f, 1.0f},
            {-0.5f,  0.5f, -0.5f, 1.0f, 0.9f, 0.1f, 1.0f},
            {-0.5f, -0.5f,  0.5f, 1.0f, 0.1f, 1.0f, 1.0f},
            { 0.5f, -0.5f,  0.5f, 0.1f, 1.0f, 1.0f, 1.0f},
            { 0.5f,  0.5f,  0.5f, 1.0f, 1.0f, 1.0f, 1.0f},
            {-0.5f,  0.5f,  0.5f, 1.0f, 0.5f, 0.1f, 1.0f},
        },
        .indices = {
            4, 5, 6, 4, 6, 7, 0, 2, 1, 0, 3, 2,
            0, 4, 7, 0, 7, 3, 1, 2, 6, 1, 6, 5,
            3, 7, 6, 3, 6, 2, 0, 1, 5, 0, 5, 4,
        },
};
