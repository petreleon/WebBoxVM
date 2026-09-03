#include "packet.h"
#include "syscall.h"

static const char render_node[] = "/dev/dri/renderD128";
static const char card_node[] = "/dev/dri/card0";
static const char serial_node[] = "/dev/ttyAMA0";
static const char pass_render[] = "WEBGPU_DEMO_PASS renderD128 capset=7 cube=8/36\n";
static const char pass_card[] = "WEBGPU_DEMO_PASS card0 capset=7 cube=8/36\n";
static const char fail_open[] = "WEBGPU_DEMO_FAIL open-drm\n";
static const char fail_context[] = "WEBGPU_DEMO_FAIL context-init\n";
static const char fail_submit[] = "WEBGPU_DEMO_FAIL execbuffer\n";

enum attempt_result {
    ATTEMPT_OK = 0,
    ATTEMPT_OPEN = -1,
    ATTEMPT_CONTEXT = -2,
    ATTEMPT_SUBMIT = -3,
};

static int try_node(const char *path)
{
    long fd = sys_open(path, O_RDWR | O_CLOEXEC);
    struct drm_virtgpu_context_set_param parameter;
    struct drm_virtgpu_context_init context;
    struct drm_virtgpu_execbuffer submit;

    if (fd < 0)
        return ATTEMPT_OPEN;
    parameter.param = VIRTGPU_CONTEXT_PARAM_CAPSET_ID;
    parameter.value = WBG3_CAPSET_ID;
    context.num_params = 1;
    context.pad = 0;
    context.ctx_set_params = (u64)&parameter;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_CONTEXT_INIT, &context) < 0) {
        sys_close(fd);
        return ATTEMPT_CONTEXT;
    }

    submit.flags = 0;
    submit.size = sizeof(cube_packet);
    submit.command = (u64)&cube_packet;
    submit.bo_handles = 0;
    submit.num_bo_handles = 0;
    submit.fence_fd = -1;
    submit.ring_idx = 0;
    submit.syncobj_stride = 0;
    submit.num_in_syncobjs = 0;
    submit.num_out_syncobjs = 0;
    submit.in_syncobjs = 0;
    submit.out_syncobjs = 0;
    if (sys_ioctl(fd, DRM_IOCTL_VIRTGPU_EXECBUFFER, &submit) < 0) {
        sys_close(fd);
        return ATTEMPT_SUBMIT;
    }
    sys_close(fd);
    return ATTEMPT_OK;
}

static void emit(const char *message, u64 length)
{
    long fd = sys_open(serial_node, O_WRONLY | O_CLOEXEC);
    u64 written = 0;
    if (fd < 0)
        fd = 1;
    while (written < length) {
        long count = sys_write(fd, message + written, length - written);
        if (count <= 0)
            break;
        written += (u64)count;
    }
    if (fd != 1)
        sys_close(fd);
}

#define EMIT(text) emit((text), sizeof(text) - 1u)

__attribute__((noreturn, section(".text.start"))) void _start(void)
{
    int render_result = try_node(render_node);
    int card_result;
    int failure;

    if (render_result == ATTEMPT_OK) {
        EMIT(pass_render);
        sys_exit(0);
    }
    card_result = try_node(card_node);
    if (card_result == ATTEMPT_OK) {
        EMIT(pass_card);
        sys_exit(0);
    }
    failure = render_result < card_result ? render_result : card_result;
    if (failure == ATTEMPT_SUBMIT)
        EMIT(fail_submit);
    else if (failure == ATTEMPT_CONTEXT)
        EMIT(fail_context);
    else
        EMIT(fail_open);
    sys_exit(-failure + 1);
}
