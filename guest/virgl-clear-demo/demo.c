#include "kms.h"
#include "ops.h"
#include "syscall.h"
#include "transfer.h"
static const char card_node[] = "/dev/dri/card0"; static const char serial_node[] = "/dev/ttyAMA0";
static const char pass[] = "VIRGL_TEXTURE_DEMO_PASS card0 capset=1 rings=2:ring1-clear mesh=2x-constant-uniform-triangle constant=121,115,134,255 blob=guest+host-map+default-shadow+renderer-local texture=10,20,30,255 linear=25,35,45,255 pair=55,65,75,255 vertex=64,64,127,255 modulate=32,32,64,255 uniform-inline-vertex=147,141,58,255 depth-less=58,102,20,255 solid-batch=0,128,64,255 depth-batch=0,0,128,255 depth-equal=128,0,0,255 depth-equal-batch=128,0,64,255 depth-mixed-batch=0,128,64,255 depth-write-mask-batch=0,128,64,255 depth-vertex-color=64,64,127,255 depth-texture=10,20,30,255 depth-texture-color=32,32,64,255 depth-material-constant=64,64,64,255\n";
static const char fail_open[] = "VIRGL_CLEAR_DEMO_FAIL open-drm\n";
static const char fail_caps[] = "VIRGL_CLEAR_DEMO_FAIL capset\n";
static const char fail_context[] = "VIRGL_CLEAR_DEMO_FAIL context-init\n";
static const char fail_resource[] = "VIRGL_CLEAR_DEMO_FAIL resource-create\n";
static const char fail_buffer_upload[] = "VIRGL_CLEAR_DEMO_FAIL buffer-upload\n";
static const char fail_vertex_state[] = "VIRGL_CLEAR_DEMO_FAIL vertex-state\n";
static const char fail_vertex_wait[] = "VIRGL_CLEAR_DEMO_FAIL vertex-wait\n";
static const char fail_buffer_copy[] = "VIRGL_CLEAR_DEMO_FAIL buffer-copy\n";
static const char fail_buffer_wait[] = "VIRGL_CLEAR_DEMO_FAIL buffer-wait\n";
static const char fail_buffer_readback[] = "VIRGL_CLEAR_DEMO_FAIL buffer-readback\n";
static const char fail_triangle_upload[] = "VIRGL_CLEAR_DEMO_FAIL triangle-upload\n";
static const char fail_triangle_submit[] = "VIRGL_CLEAR_DEMO_FAIL triangle-submit\n";
static const char fail_triangle_wait[] = "VIRGL_CLEAR_DEMO_FAIL triangle-wait\n";
static const char fail_triangle_readback[] = "VIRGL_CLEAR_DEMO_FAIL triangle-readback\n";
static const char fail_transfer[] = "VIRGL_CLEAR_DEMO_FAIL transfer-upload\n";
static const char fail_copy_upload[] = "VIRGL_CLEAR_DEMO_FAIL copy-upload\n";
static const char fail_copy_submit[] = "VIRGL_CLEAR_DEMO_FAIL copy-submit\n";
static const char fail_copy_wait[] = "VIRGL_CLEAR_DEMO_FAIL copy-wait\n";
static const char fail_copy_readback[] = "VIRGL_CLEAR_DEMO_FAIL copy-readback\n";
static const char fail_submit[] = "VIRGL_CLEAR_DEMO_FAIL execbuffer\n";
static const char fail_wait[] = "VIRGL_CLEAR_DEMO_FAIL completion-wait\n";
static const char fail_readback[] = "VIRGL_CLEAR_DEMO_FAIL transfer-readback\n";
static const char fail_texture_pair[] = "VIRGL_CLEAR_DEMO_FAIL texture-pair\n";
static const char fail_vertex_color[] = "VIRGL_CLEAR_DEMO_FAIL vertex-color\n";
static const char fail_texture_color[] = "VIRGL_CLEAR_DEMO_FAIL texture-color\n";
static const char fail_uniform[] = "VIRGL_CLEAR_DEMO_FAIL uniform-buffer\n"; static const char fail_depth[] = "VIRGL_CLEAR_DEMO_FAIL depth-less\n"; static const char fail_batch[] = "VIRGL_CLEAR_DEMO_FAIL solid-batch\n"; static const char fail_depth_batch[] = "VIRGL_CLEAR_DEMO_FAIL depth-batch\n"; static const char fail_depth_equal[] = "VIRGL_CLEAR_DEMO_FAIL depth-equal\n"; static const char fail_depth_equal_batch[] = "VIRGL_CLEAR_DEMO_FAIL depth-equal-batch\n"; static const char fail_depth_mixed_batch[] = "VIRGL_CLEAR_DEMO_FAIL depth-mixed-batch\n"; static const char fail_depth_write_mask_batch[] = "VIRGL_CLEAR_DEMO_FAIL depth-write-mask-batch\n"; static const char fail_depth_vertex_color[] = "VIRGL_CLEAR_DEMO_FAIL depth-vertex-color\n"; static const char fail_depth_texture[] = "VIRGL_CLEAR_DEMO_FAIL depth-texture\n"; static const char fail_depth_texture_color[] = "VIRGL_CLEAR_DEMO_FAIL depth-texture-color\n"; static const char fail_material_batch[] = "VIRGL_CLEAR_DEMO_FAIL depth-material-constant\n";
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
static void emit_kms_failure(int step)
{
    char message[] = "VIRGL_CLEAR_DEMO_FAIL kms-step=00\n";
    message[31] = (char)('0' + step / 10);
    message[32] = (char)('0' + step % 10);
    emit(message, sizeof(message) - 1u);
}
__attribute__((noreturn, section(".text.start"))) void _start(void) {
    struct virgl_resources resources = {0};
    long fd = sys_open(card_node, O_RDWR | O_CLOEXEC);
    int stage = 0;
    if (fd < 0)
        stage = 1;
    else if ((stage = virgl_setup(fd, &resources)) == 0) {
        if (virgl_upload_vertex_buffer(fd, resources.vertex_source_bo) != 0)
            stage = 30;
        else if (virgl_submit_vertex_input(fd, resources.vertex_source_bo,
                                           resources.vertex_source_resource) != 0)
            stage = 31;
        else if (virgl_wait_for_resource(fd, resources.vertex_source_bo) != 0)
            stage = 32;
        else if (virgl_submit_buffer_copy(fd, resources.vertex_source_bo,
                                          resources.vertex_source_resource,
                                          resources.vertex_destination_bo,
                                          resources.vertex_destination_resource) != 0)
            stage = 33;
        else if (virgl_wait_for_resource(fd, resources.vertex_destination_bo) != 0)
            stage = 34;
        else if (virgl_readback_vertex_buffer(fd, resources.vertex_destination_bo) != 0)
            stage = 35;
        else {
            int kms = kms_configure_scanout(fd, resources.scanout_bo);
            if (kms != 0)
                stage = 5 - kms;
            else if (virgl_upload_pattern(fd, resources.scanout_bo) != 0)
                stage = 5;
            else if (virgl_upload_copy_source(fd, resources.copy_source_bo) != 0)
                stage = 16;
            else if (virgl_submit_copy(fd, resources.copy_source_bo,
                                       resources.copy_source_resource,
                                       resources.copy_destination_bo,
                                       resources.copy_destination_resource) != 0)
                stage = 17;
            else if (virgl_wait_for_resource(fd, resources.copy_destination_bo) != 0)
                stage = 18;
            else if (virgl_readback_copy_destination(fd, resources.copy_destination_bo) != 0)
                stage = 19;
            else {
                kms = kms_configure_scanout(fd, resources.scanout_bo);
                if (kms != 0)
                    stage = 5 - kms;
                else if (virgl_submit_clear(fd, resources.scanout_bo,
                                            resources.scanout_resource) != 0)
                    stage = 20;
                else if (virgl_wait_for_resource(fd, resources.scanout_bo) != 0)
                    stage = 21;
                else if (virgl_readback_clear(fd, resources.scanout_bo) != 0)
                    stage = 22;
                else if ((stage = virgl_run_triangle(fd, &resources)) != 0)
                    stage += 35;
                else if ((stage = virgl_run_textured_triangle(fd, &resources)) != 0)
                    stage += 40;
                else if ((stage = virgl_run_texture_pair(fd, &resources)) != 0)
                    stage += 44;
                else if ((stage = virgl_run_vertex_color_triangle(fd, &resources)) != 0)
                    stage += 49;
                else if ((stage = virgl_run_texture_color_triangle(fd, &resources)) != 0)
                    stage += 53;
                else if ((stage = virgl_run_uniform_triangle(fd, &resources)) != 0)
                    stage += 57;
                else if ((stage = virgl_run_depth_triangle(fd, &resources)) != 0) stage += 62;
                else if ((stage = virgl_run_solid_batch(fd, &resources)) != 0) stage += 66;
                else if ((stage = virgl_run_depth_batch(fd, &resources)) != 0) stage += 70; else if ((stage = virgl_run_depth_equal(fd, &resources)) != 0) stage += 74; else if ((stage = virgl_run_depth_equal_batch(fd, &resources)) != 0) stage += 78; else if ((stage = virgl_run_depth_mixed_batch(fd, &resources)) != 0) stage += 82; else if ((stage = virgl_run_depth_write_mask_batch(fd, &resources)) != 0) stage += 86; else if ((stage = virgl_run_depth_vertex_color_triangle(fd, &resources)) != 0) stage += 90; else if ((stage = virgl_run_depth_textured_triangle(fd, &resources)) != 0) stage += 94; else if ((stage = virgl_run_depth_texture_color_triangle(fd, &resources)) != 0) stage += 98; else if ((stage = virgl_run_material_batch(fd, &resources)) != 0) stage += 102;
            }
        }
    }
    if (stage == 0)
        EMIT(pass);
    else if (stage == 1)
        EMIT(fail_open);
    else if (stage == 2)
        EMIT(fail_caps);
    else if (stage == 3)
        EMIT(fail_context);
    else if (stage == 4)
        EMIT(fail_resource);
    else if (stage == 5)
        EMIT(fail_transfer);
    else if (stage >= 6 && stage <= 15)
        emit_kms_failure(stage - 5);
    else if (stage == 16)
        EMIT(fail_copy_upload);
    else if (stage == 17)
        EMIT(fail_copy_submit);
    else if (stage == 18)
        EMIT(fail_copy_wait);
    else if (stage == 19)
        EMIT(fail_copy_readback);
    else if (stage == 20)
        EMIT(fail_submit);
    else if (stage == 21)
        EMIT(fail_wait);
    else if (stage == 30)
        EMIT(fail_buffer_upload);
    else if (stage == 31)
        EMIT(fail_vertex_state);
    else if (stage == 32)
        EMIT(fail_vertex_wait);
    else if (stage == 33)
        EMIT(fail_buffer_copy);
    else if (stage == 34)
        EMIT(fail_buffer_wait);
    else if (stage == 35)
        EMIT(fail_buffer_readback);
    else if (stage == 36)
        EMIT(fail_triangle_upload);
    else if (stage == 37)
        EMIT(fail_triangle_submit);
    else if (stage == 38)
        EMIT(fail_triangle_wait);
    else if (stage == 39)
        EMIT(fail_triangle_readback);
    else if (stage >= 41 && stage <= 44)
        EMIT(fail_triangle_readback);
    else if (stage >= 45 && stage <= 49) EMIT(fail_texture_pair);
    else if (stage >= 50 && stage <= 53) EMIT(fail_vertex_color);
    else if (stage >= 54 && stage <= 57) EMIT(fail_texture_color);
    else if (stage >= 58 && stage <= 62) EMIT(fail_uniform); else if (stage >= 63 && stage <= 66) EMIT(fail_depth); else if (stage >= 67 && stage <= 70) EMIT(fail_batch); else if (stage >= 71 && stage <= 74) EMIT(fail_depth_batch); else if (stage >= 75 && stage <= 78) EMIT(fail_depth_equal); else if (stage >= 79 && stage <= 82) EMIT(fail_depth_equal_batch); else if (stage >= 83 && stage <= 86) EMIT(fail_depth_mixed_batch); else if (stage >= 87 && stage <= 90) EMIT(fail_depth_write_mask_batch); else if (stage >= 91 && stage <= 94) EMIT(fail_depth_vertex_color); else if (stage >= 95 && stage <= 98) EMIT(fail_depth_texture); else if (stage >= 99 && stage <= 102) EMIT(fail_depth_texture_color); else if (stage >= 103 && stage <= 106) EMIT(fail_material_batch);
    else EMIT(fail_readback);
    if (fd >= 0) { sys_close(fd); } sys_exit(0);
}
