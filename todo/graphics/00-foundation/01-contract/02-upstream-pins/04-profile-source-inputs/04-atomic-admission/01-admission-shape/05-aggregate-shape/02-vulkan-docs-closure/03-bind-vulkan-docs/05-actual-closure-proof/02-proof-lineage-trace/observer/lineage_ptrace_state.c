#define _GNU_SOURCE
#include "lineage_ptrace_state.h"

#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <string.h>
#include <sys/ptrace.h>
#include <sys/syscall.h>
#include <unistd.h>

#define ARGV "797b8f932ac58c625cfb33c986ba1e4b5eb471598cb3847781a4eae40f8c21fb"

static struct lpc_process *process(struct lpc_state *state, pid_t pid) {
    int index;
    for (index = 0; index < LPC_PROCS; ++index)
        if (state->processes[index].used && state->processes[index].pid == pid) return &state->processes[index];
    return NULL;
}

static int emit(struct lpc_state *state, const char *kind, pid_t pid, const char *tail) {
    return fprintf(state->out, "{\"kind\":\"%s\",\"pid\":%d%s}\n", kind, (int)pid, tail) < 0 ? -1 : 0;
}

static int emit_fd(struct lpc_state *state, const char *kind, pid_t pid, int number) {
    return fprintf(state->out, "{\"kind\":\"%s\",\"pid\":%d,\"fd\":%d}\n", kind, (int)pid, number) < 0 ? -1 : 0;
}

static int emit_process(struct lpc_state *state, pid_t pid, pid_t parent) {
    if (parent) return fprintf(state->out, "{\"kind\":\"process\",\"pid\":%d,\"parent\":%d,\"argv_sha256\":\"%s\"}\n", (int)pid, (int)parent, ARGV) < 0 ? -1 : 0;
    return fprintf(state->out, "{\"kind\":\"process\",\"pid\":%d,\"parent\":null,\"argv_sha256\":\"%s\"}\n", (int)pid, ARGV) < 0 ? -1 : 0;
}

static int live(const struct lpc_process *item) {
    int index;
    for (index = 0; index < LPC_FDS; ++index) if (item->fds[index].used) return 1;
    return 0;
}

static struct lpc_fd *fd(struct lpc_process *item, long number, int create) {
    int index;
    for (index = 0; index < LPC_FDS; ++index)
        if (item->fds[index].used && item->fds[index].number == number) return &item->fds[index];
    if (!create) return NULL;
    for (index = 0; index < LPC_FDS; ++index)
        if (!item->fds[index].used) return &item->fds[index];
    return NULL;
}

static int root(const char *path) {
    if (!strncmp(path, "/vulkan/", 8) && path[8]) return 1;
    if (!strncmp(path, "/work/temporary/", 16) && path[16]) return 2;
    if (!strncmp(path, "/work/generated/", 16) && path[16]) return 3;
    return 0;
}

static int safe(const char *path) {
    const unsigned char *at = (const unsigned char *)path; size_t length = strlen(path);
    for (; *at; ++at) if (!(isalnum(*at) || *at == '/' || *at == '.' || *at == '_' || *at == '-')) return 0;
    return length > 2 && strcmp(path + length - 2, "/.") && strcmp(path + length - 3, "/..") && !strstr(path, "//") && !strstr(path, "/../") && !strstr(path, "/./");
}

static int read_path(pid_t pid, unsigned long long address, char *path) {
    size_t offset;
    for (offset = 0; offset < LPC_PATH; offset += sizeof(long)) {
        long word; size_t index;
        errno = 0; word = ptrace(PTRACE_PEEKDATA, pid, (char *)(unsigned long)address + offset, NULL);
        if (word == -1 && errno) return -1;
        for (index = 0; index < sizeof(word) && offset + index < LPC_PATH; ++index) {
            path[offset + index] = ((char *)&word)[index];
            if (!path[offset + index]) return safe(path) ? root(path) : -1;
        }
    }
    return -1;
}

static int pending(struct lpc_process *item, enum lpc_call kind) {
    if (item == NULL || item->pending.kind != LPC_NONE) return -1;
    item->pending.kind = kind;
    return 0;
}

static int expected_open(int root_kind, long flags) {
    long input = O_RDONLY | O_CLOEXEC | O_NOFOLLOW;
    long output = O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW;
    return flags == (root_kind == 2 ? output : input);
}

static int fixture_path(int root_kind, const char *path) {
    return (root_kind == 1 && !strcmp(path, "/vulkan/raw.adoc")) || (root_kind == 2 && !strcmp(path, "/work/temporary/out.tmp")) ||
           (root_kind == 3 && !strcmp(path, "/work/generated/out.adoc"));
}

void lpc_init(struct lpc_state *state, FILE *out) { memset(state, 0, sizeof(*state)); state->out = out; }

int lpc_start(struct lpc_state *state, pid_t pid, pid_t parent) {
    int index; struct lpc_process *item;
    if (process(state, pid)) return -1;
    for (index = 0; index < LPC_PROCS; ++index) if (!state->processes[index].used) break;
    if (index == LPC_PROCS) return -1;
    item = &state->processes[index]; memset(item, 0, sizeof(*item)); item->used = 1; item->pid = pid; item->parent = parent;
    return emit_process(state, pid, parent);
}

int lpc_fork(struct lpc_state *state, pid_t parent, pid_t child) {
    struct lpc_process *item = process(state, parent);
    return item == NULL || live(item) || item->pending.kind ? -1 : lpc_start(state, child, parent);
}

int lpc_entry(struct lpc_state *state, pid_t pid, long number, const uint64_t *args) {
    struct lpc_process *item = process(state, pid); int first, second;
    if (item == NULL) return -1;
    if (number == SYS_openat) {
        first = read_path(pid, args[1], item->pending.old);
        if (first <= 0) return first ? -1 : live(item) || item->pending.kind ? -1 : 0;
        if ((long)args[0] != AT_FDCWD || !fixture_path(first, item->pending.old) || !expected_open(first, (long)args[2]) ||
            (first == 2 ? args[3] != 0600 : args[3] != 0) || pending(item, LPC_OPEN)) return -1;
        item->pending.flags = (long)(args[2] & O_ACCMODE); return 0;
    }
    if (number == SYS_read || number == SYS_write || number == SYS_close) {
        struct lpc_fd *entry = fd(item, (long)args[0], 0); enum lpc_call call = number == SYS_read ? LPC_READ : number == SYS_write ? LPC_WRITE : LPC_CLOSE;
        if (entry == NULL) return live(item) || item->pending.kind ? -1 : 0;
        if ((call == LPC_READ && entry->mode != O_RDONLY) || (call == LPC_WRITE && entry->mode != O_WRONLY) || pending(item, call)) return -1;
        if ((call == LPC_READ && args[2] != 64) || (call == LPC_WRITE && args[2] != 3)) return -1;
        item->pending.fd = (long)args[0]; item->pending.flags = (long)args[2]; return 0;
    }
    if (number == SYS_renameat) {
        first = read_path(pid, args[1], item->pending.old); second = read_path(pid, args[3], item->pending.new);
        if (!first && !second) return live(item) || item->pending.kind ? -1 : 0;
        if (first != 2 || second != 3 || !fixture_path(first, item->pending.old) || !fixture_path(second, item->pending.new) ||
            (long)args[0] != AT_FDCWD || (long)args[2] != AT_FDCWD || pending(item, LPC_RENAME)) return -1;
        return 0;
    }
    return live(item) || item->pending.kind ? -1 : 0;
}

int lpc_exit(struct lpc_state *state, pid_t pid, long result) {
    struct lpc_process *item = process(state, pid); struct lpc_fd *entry; const char *mode;
    if (item == NULL || item->pending.kind == LPC_NONE) return item == NULL ? -1 : 0;
    if (result < 0) return -1;
    if (item->pending.kind == LPC_OPEN) {
        if (result > 65535 || (entry = fd(item, result, 1)) == NULL) return -1;
        entry->used = 1; entry->number = (int)result; entry->mode = item->pending.flags; strcpy(entry->path, item->pending.old);
        mode = entry->mode == O_RDONLY ? "read" : "write";
        if (fprintf(state->out, "{\"kind\":\"open\",\"pid\":%d,\"fd\":%ld,\"path\":\"%s\",\"mode\":\"%s\"}\n", (int)pid, result, entry->path, mode) < 0) return -1;
    } else if (item->pending.kind == LPC_READ || item->pending.kind == LPC_WRITE) {
        entry = fd(item, item->pending.fd, 0); if (entry == NULL || result != 3 || result > item->pending.flags) return -1;
        if (emit_fd(state, item->pending.kind == LPC_READ ? "read" : "write", pid, entry->number)) return -1;
    } else if (item->pending.kind == LPC_CLOSE) {
        entry = fd(item, item->pending.fd, 0); if (entry == NULL || result != 0 || emit_fd(state, "close", pid, entry->number)) return -1;
        entry->used = 0;
    } else if (result != 0 || fprintf(state->out, "{\"kind\":\"rename\",\"pid\":%d,\"old\":\"%s\",\"new\":\"%s\"}\n", (int)pid, item->pending.old, item->pending.new) < 0) return -1;
    item->pending.kind = LPC_NONE; return 0;
}

int lpc_exec(struct lpc_state *state, pid_t pid) { struct lpc_process *item = process(state, pid); return item == NULL || live(item) || item->pending.kind ? -1 : emit(state, "exec", pid, ",\"argv_sha256\":\"" ARGV "\""); }
int lpc_exit_process(struct lpc_state *state, pid_t pid, int code) { struct lpc_process *item = process(state, pid); if (item == NULL || live(item) || item->pending.kind || code) return -1; item->used = 0; return emit(state, "exit", pid, ""); }
int lpc_empty(const struct lpc_state *state) { int index; for (index = 0; index < LPC_PROCS; ++index) if (state->processes[index].used) return 0; return 1; }
int lpc_known(const struct lpc_state *state, pid_t pid) { int index; for (index = 0; index < LPC_PROCS; ++index) if (state->processes[index].used && state->processes[index].pid == pid) return 1; return 0; }
void lpc_kill(const struct lpc_state *state) { int index; for (index = 0; index < LPC_PROCS; ++index) if (state->processes[index].used) (void)kill(state->processes[index].pid, SIGKILL); }
