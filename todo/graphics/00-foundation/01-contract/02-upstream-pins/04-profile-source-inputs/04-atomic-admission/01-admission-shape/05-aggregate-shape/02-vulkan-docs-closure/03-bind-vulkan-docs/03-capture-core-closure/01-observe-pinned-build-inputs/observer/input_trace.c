#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <unistd.h>

typedef ssize_t (*read_fn)(int, void *, size_t);
typedef ssize_t (*pread_fn)(int, void *, size_t, off_t);
typedef ssize_t (*readv_fn)(int, const struct iovec *, int);
typedef void *(*mmap_fn)(void *, size_t, int, int, int, off_t);
typedef size_t (*fread_fn)(void *, size_t, size_t, FILE *);
typedef char *(*fgets_fn)(char *, int, FILE *);
typedef ssize_t (*copy_fn)(int, off_t *, int, off_t *, size_t, unsigned int);
typedef ssize_t (*sendfile_fn)(int, int, off_t *, size_t);
typedef ssize_t (*splice_fn)(int, loff_t *, int, loff_t *, size_t, unsigned int);

static int out = -1;
static __thread int busy;
static read_fn real_read;
static pread_fn real_pread;
static readv_fn real_readv;
static mmap_fn real_mmap;
static fread_fn real_fread;
static fgets_fn real_fgets;
static copy_fn real_copy;
static sendfile_fn real_sendfile;
static splice_fn real_splice;

static void raw(const char *value, size_t length) {
    if (out >= 0) (void) syscall(SYS_write, out, value, length);
}

static int keep(const char *path) {
    return !strncmp(path, "/vulkan/", 8) || !strncmp(path, "/work/generated/", 16);
}

static size_t hex(char *output, size_t maximum, const char *input, size_t length) {
    static const char digits[] = "0123456789abcdef";
    size_t index;
    if (maximum < length * 2 + 1) return 0;
    for (index = 0; index < length; ++index) {
        output[index * 2] = digits[((unsigned char) input[index]) >> 4];
        output[index * 2 + 1] = digits[((unsigned char) input[index]) & 15];
    }
    output[length * 2] = '\0';
    return length * 2;
}

static void note(const char *kind, int fd, ssize_t bytes) {
    char link[32], path[PATH_MAX], encoded[PATH_MAX * 2 + 1], line[PATH_MAX * 2 + 256];
    ssize_t length;
    int rendered; struct stat state;
    long long observed_size = -1, observed_mtime_ns = -1;
    if (busy || out < 0 || fd < 0) return;
    busy = 1;
    (void) snprintf(link, sizeof link, "/proc/self/fd/%d", fd);
    length = syscall(SYS_readlinkat, AT_FDCWD, link, path, sizeof path - 1);
    if (length > 0) {
        path[length] = '\0';
        if (keep(path) && hex(encoded, sizeof encoded, path, (size_t) length)) {
            if (syscall(SYS_fstat, fd, &state) == 0) {
                observed_size = (long long) state.st_size;
                observed_mtime_ns = (long long) state.st_mtim.tv_sec * 1000000000LL + state.st_mtim.tv_nsec;
            }
            rendered = snprintf(line, sizeof line,
                "{\"kind\":\"%s\",\"pid\":%ld,\"bytes\":%zd,\"path_hex\":\"%s\",\"observed_size\":%lld,\"observed_mtime_ns\":%lld}\n",
                kind, (long) getpid(), bytes, encoded, observed_size, observed_mtime_ns);
            if (rendered > 0 && (size_t) rendered < sizeof line) raw(line, (size_t) rendered);
        }
    }
    busy = 0;
}

static void start(void) {
    char command[16384], encoded[sizeof command * 2 + 1], line[sizeof encoded + 96];
    const char *path = getenv("WEBBOXVM_TRACE");
    ssize_t length;
    int fd, rendered;
    if (!path) return;
    out = syscall(SYS_openat, AT_FDCWD, path, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0600);
    fd = syscall(SYS_openat, AT_FDCWD, "/proc/self/cmdline", O_RDONLY | O_CLOEXEC);
    length = fd < 0 ? -1 : syscall(SYS_read, fd, command, sizeof command);
    if (fd >= 0) (void) syscall(SYS_close, fd);
    if (length > 0 && hex(encoded, sizeof encoded, command, (size_t) length)) {
        rendered = snprintf(line, sizeof line, "{\"kind\":\"start\",\"pid\":%ld,\"argv_hex\":\"%s\"}\n",
            (long) getpid(), encoded);
        if (rendered > 0 && (size_t) rendered < sizeof line) raw(line, (size_t) rendered);
    }
}

static void init(void) {
    if (busy || real_read) return;
    busy = 1;
    real_read = (read_fn) dlsym(RTLD_NEXT, "read");
    real_pread = (pread_fn) dlsym(RTLD_NEXT, "pread64");
    real_readv = (readv_fn) dlsym(RTLD_NEXT, "readv");
    real_mmap = (mmap_fn) dlsym(RTLD_NEXT, "mmap");
    real_fread = (fread_fn) dlsym(RTLD_NEXT, "fread");
    real_fgets = (fgets_fn) dlsym(RTLD_NEXT, "fgets");
    real_copy = (copy_fn) dlsym(RTLD_NEXT, "copy_file_range");
    real_sendfile = (sendfile_fn) dlsym(RTLD_NEXT, "sendfile");
    real_splice = (splice_fn) dlsym(RTLD_NEXT, "splice");
    busy = 0;
}

__attribute__((constructor)) static void initialize(void) { start(); init(); }

ssize_t read(int fd, void *buffer, size_t count) {
    ssize_t result; init(); result = real_read ? real_read(fd, buffer, count) : syscall(SYS_read, fd, buffer, count);
    if (result > 0) note("read", fd, result);
    return result;
}

ssize_t pread64(int fd, void *buffer, size_t count, off_t offset) {
    ssize_t result; init(); result = real_pread ? real_pread(fd, buffer, count, offset) : syscall(SYS_pread64, fd, buffer, count, offset);
    if (result > 0) note("pread", fd, result);
    return result;
}

ssize_t readv(int fd, const struct iovec *iov, int count) {
    ssize_t result; init(); result = real_readv ? real_readv(fd, iov, count) : syscall(SYS_readv, fd, iov, count);
    if (result > 0) note("readv", fd, result);
    return result;
}

void *mmap(void *address, size_t length, int protection, int flags, int fd, off_t offset) {
    void *result; init(); result = real_mmap ? real_mmap(address, length, protection, flags, fd, offset) :
        (void *) syscall(SYS_mmap, address, length, protection, flags, fd, offset);
    if (result != MAP_FAILED && (protection & PROT_READ)) note("mmap", fd, (ssize_t) length);
    return result;
}

size_t fread(void *buffer, size_t size, size_t count, FILE *stream) {
    size_t result; init(); if (!real_fread) abort(); result = real_fread(buffer, size, count, stream);
    if (result && size <= (size_t) SSIZE_MAX / result) note("fread", fileno(stream), (ssize_t) (result * size));
    return result;
}

char *fgets(char *buffer, int size, FILE *stream) {
    char *result; init(); if (!real_fgets) abort(); result = real_fgets(buffer, size, stream);
    if (result && strlen(buffer)) note("fgets", fileno(stream), (ssize_t) strlen(buffer));
    return result;
}

ssize_t copy_file_range(int input, off_t *input_offset, int output, off_t *output_offset, size_t length, unsigned int flags) {
    ssize_t result; init(); result = real_copy ? real_copy(input, input_offset, output, output_offset, length, flags) :
        syscall(SYS_copy_file_range, input, input_offset, output, output_offset, length, flags);
    if (result > 0) note("copy", input, result);
    return result;
}

ssize_t sendfile(int output, int input, off_t *offset, size_t length) {
    ssize_t result; init(); result = real_sendfile ? real_sendfile(output, input, offset, length) : syscall(SYS_sendfile, output, input, offset, length);
    if (result > 0) note("sendfile", input, result);
    return result;
}

ssize_t splice(int input, loff_t *input_offset, int output, loff_t *output_offset, size_t length, unsigned int flags) {
    ssize_t result; init(); result = real_splice ? real_splice(input, input_offset, output, output_offset, length, flags) :
        syscall(SYS_splice, input, input_offset, output, output_offset, length, flags);
    if (result > 0) note("splice", input, result);
    return result;
}
