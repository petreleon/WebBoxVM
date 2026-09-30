#define _POSIX_C_SOURCE 200809L
#include "probe.h"
#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <GLES3/gl3.h>
#include <gbm.h>

#define LABEL "I01_MESA_GLES_STARTUP"
#define CHECK(condition, stage) do { if (!(condition)) { \
    fprintf(stderr, LABEL "_FAIL stage=%s egl=0x%x errno=%d\n", \
            stage, eglGetError(), errno); goto done; } } while (0)

static int extension(const char *extensions, const char *name)
{
    size_t length = strlen(name);
    if (!extensions) return 0;
    for (const char *p = extensions; (p = strstr(p, name)); p += length)
        if ((p == extensions || p[-1] == ' ') && (p[length] == ' ' || !p[length])) return 1;
    return 0;
}

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    if (argc > 2) { fprintf(stderr, "usage: %s [/dev/dri/renderD128]\n", argv[0]); return 2; }
    if (probe_environment(LABEL)) return 1;
    int fd = probe_drm_open(LABEL, argc == 2 ? argv[1] : "/dev/dri/renderD128");
    if (fd < 0) return 1;
    int result = 1, current = 0, initialized = 0;
    struct gbm_device *gbm = NULL;
    EGLDisplay display = EGL_NO_DISPLAY;
    EGLContext context = EGL_NO_CONTEXT;
    EGLConfig config;
    EGLint count = 0, egl_major = 0, egl_minor = 0;
    const EGLint attributes[] = {EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
                                EGL_SURFACE_TYPE, 0, EGL_NONE};
    const EGLint context_attributes[] = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE};
    CHECK((gbm = gbm_create_device(fd)), "gbm_create_device");
    display = eglGetPlatformDisplay(EGL_PLATFORM_GBM_KHR, gbm, NULL);
    CHECK(display != EGL_NO_DISPLAY, "eglGetPlatformDisplay_GBM");
    CHECK(eglInitialize(display, &egl_major, &egl_minor), "eglInitialize");
    initialized = 1;
    printf("EGL_API=%d.%d\n", egl_major, egl_minor);
    const char *egl_vendor = eglQueryString(display, EGL_VENDOR);
    const char *extensions = eglQueryString(display, EGL_EXTENSIONS);
    CHECK(egl_vendor && !probe_software(egl_vendor), "egl_vendor");
    probe_text("EGL_VENDOR", egl_vendor);
    CHECK(extension(extensions, "EGL_MESA_query_driver"), "EGL_MESA_query_driver");
    PFNEGLGETDISPLAYDRIVERNAMEPROC get_driver =
        (PFNEGLGETDISPLAYDRIVERNAMEPROC)eglGetProcAddress("eglGetDisplayDriverName");
    CHECK(get_driver, "eglGetDisplayDriverName_symbol");
    const char *driver = get_driver(display);
    CHECK(driver && !strcmp(driver, "virtio_gpu"), "egl_driver_identity");
    probe_text("EGL_DRIVER", driver);
    CHECK(extension(extensions, "EGL_KHR_surfaceless_context"), "surfaceless_context");
    CHECK(eglBindAPI(EGL_OPENGL_ES_API), "eglBindAPI_GLES");
    CHECK(eglChooseConfig(display, attributes, &config, 1, &count) && count == 1,
          "eglChooseConfig_GLES3");
    context = eglCreateContext(display, config, EGL_NO_CONTEXT, context_attributes);
    CHECK(context != EGL_NO_CONTEXT, "eglCreateContext_GLES3");
    CHECK(eglMakeCurrent(display, EGL_NO_SURFACE, EGL_NO_SURFACE, context), "eglMakeCurrent");
    current = 1;
    const char *vendor = (const char *)glGetString(GL_VENDOR);
    const char *renderer = (const char *)glGetString(GL_RENDERER);
    const char *version = (const char *)glGetString(GL_VERSION);
    GLint major = 0, minor = 0;
    glGetIntegerv(GL_MAJOR_VERSION, &major);
    glGetIntegerv(GL_MINOR_VERSION, &minor);
    GLenum gl_error = glGetError();
    if (gl_error != GL_NO_ERROR) {
        fprintf(stderr, LABEL "_FAIL stage=gl_identity_queries gl=0x%x\n", gl_error);
        goto done;
    }
    CHECK(vendor && *vendor && renderer && *renderer && version && *version,
          "gl_identity_strings");
    probe_text("GL_VENDOR", vendor);
    probe_text("GL_RENDERER", renderer);
    probe_text("GL_VERSION", version);
    printf("GLES_API=%d.%d\n", major, minor);
    CHECK(!probe_software(renderer) && !probe_software(vendor), "software_renderer");
    CHECK(probe_mesa_version(version) && major >= 3, "pinned_mesa_GLES3");
    result = 0;
done:
    if (current && !eglMakeCurrent(display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT)) {
        fprintf(stderr, LABEL "_FAIL stage=eglMakeCurrent_release egl=0x%x\n", eglGetError());
        result = 1;
    }
    if (context != EGL_NO_CONTEXT && !eglDestroyContext(display, context)) {
        fprintf(stderr, LABEL "_FAIL stage=eglDestroyContext egl=0x%x\n", eglGetError()); result = 1;
    }
    if (initialized && !eglTerminate(display)) {
        fprintf(stderr, LABEL "_FAIL stage=eglTerminate egl=0x%x\n", eglGetError()); result = 1;
    }
    if (gbm) gbm_device_destroy(gbm);
    if (close(fd)) { fprintf(stderr, LABEL "_FAIL stage=close errno=%d\n", errno); result = 1; }
    if (!result) puts(LABEL "_PASS");
    return result;
}
