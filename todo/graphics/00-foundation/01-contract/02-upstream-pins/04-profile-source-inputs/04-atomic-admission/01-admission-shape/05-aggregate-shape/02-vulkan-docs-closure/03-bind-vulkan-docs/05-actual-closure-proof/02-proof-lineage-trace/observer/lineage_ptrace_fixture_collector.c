#define _GNU_SOURCE
#include "lineage_ptrace_state.h"

#include <errno.h>
#include <fcntl.h>
#include <linux/audit.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/ptrace.h>
#include <sys/prctl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef PTRACE_GET_SYSCALL_INFO
#define PTRACE_GET_SYSCALL_INFO 0x420e
#endif

struct syscall_info {
    uint8_t op, pad[3]; uint32_t arch; uint64_t instruction_pointer, stack_pointer;
    union { struct { uint64_t nr, args[6]; } entry; struct { int64_t rval; uint8_t is_error; } exit; };
};

struct image { struct stat stat; unsigned char bytes[3]; };

static int die(const struct lpc_state *state, const char *stage) {
    lpc_kill(state); printf("{\"kind\":\"terminal\",\"status\":\"blocked\",\"stage\":\"%s\"}\n", stage);
    (void)fflush(stdout); return 77;
}

static int resume(pid_t pid, int signal) {
    return ptrace(PTRACE_SYSCALL, pid, NULL, (void *)(long)signal) ? -1 : 0;
}

static int syscall_stop(pid_t pid, struct syscall_info *info) {
    memset(info, 0, sizeof(*info));
    return ptrace(PTRACE_GET_SYSCALL_INFO, pid, sizeof(*info), info) < 0 || info->arch != AUDIT_ARCH_X86_64 ? -1 : 0;
}

static void child(const char *fixture) {
    int null = open("/dev/null", O_RDWR);
    if (null < 0 || dup2(null, STDIN_FILENO) < 0 || dup2(null, STDOUT_FILENO) < 0 || dup2(null, STDERR_FILENO) < 0) _exit(81);
    if (null > STDERR_FILENO) close(null);
    if (ptrace(PTRACE_TRACEME, 0, NULL, NULL)) _exit(82);
    raise(SIGSTOP); execl(fixture, fixture, (char *)NULL); _exit(83);
}

static int stable(const struct stat *before, const struct stat *after) {
    return before->st_dev == after->st_dev && before->st_ino == after->st_ino && before->st_mode == after->st_mode && before->st_nlink == after->st_nlink && before->st_size == after->st_size && before->st_mtim.tv_sec == after->st_mtim.tv_sec && before->st_mtim.tv_nsec == after->st_mtim.tv_nsec && before->st_ctim.tv_sec == after->st_ctim.tv_sec && before->st_ctim.tv_nsec == after->st_ctim.tv_nsec;
}

static int image(const char *path, struct image *value) {
    struct stat after; unsigned char extra; int descriptor = open(path, O_RDONLY | O_NOFOLLOW | O_CLOEXEC);
    if (descriptor < 0) return -1;
    if (fstat(descriptor, &value->stat) || !S_ISREG(value->stat.st_mode) || value->stat.st_nlink != 1 || value->stat.st_size != (off_t)sizeof(value->bytes)) { (void)close(descriptor); return -1; }
    if (read(descriptor, value->bytes, sizeof(value->bytes)) != (ssize_t)sizeof(value->bytes) || read(descriptor, &extra, 1) != 0 || fstat(descriptor, &after)) { (void)close(descriptor); return -1; }
    return close(descriptor) || !stable(&value->stat, &after) ? -1 : 0;
}

static int snapshot(const char *path, const struct image *value) {
    return printf("{\"kind\":\"snapshot\",\"path\":\"%s\",\"hex\":\"%02x%02x%02x\"}\n", path, value->bytes[0], value->bytes[1], value->bytes[2]) < 0 ? -1 : 0;
}

int main(int argc, char **argv) {
    const unsigned long options = PTRACE_O_TRACESYSGOOD | PTRACE_O_TRACEFORK | PTRACE_O_TRACEVFORK | PTRACE_O_TRACECLONE | PTRACE_O_TRACEEXEC | PTRACE_O_EXITKILL;
    struct lpc_state state; struct syscall_info info; struct image raw, after, output; int status; pid_t initial, waited;
    setvbuf(stdout, NULL, _IONBF, 0); lpc_init(&state, stdout);
    if (argc != 2 || strcmp(argv[1], "/work/fixture")) return die(&state, "fixture-argv");
    initial = fork(); if (initial < 0) return die(&state, "fork");
    if (!initial) child(argv[1]);
    if (waitpid(initial, &status, WUNTRACED) != initial || !WIFSTOPPED(status) || WSTOPSIG(status) != SIGSTOP) return die(&state, "initial-stop");
    if (prctl(PR_SET_DUMPABLE, 0)) return die(&state, "collector-undumpable");
    if (image("/vulkan/raw.adoc", &raw)) return die(&state, "pre-run-snapshot");
    if (lpc_start(&state, initial, 0) || ptrace(PTRACE_SETOPTIONS, initial, NULL, (void *)options)) return die(&state, "initial-options");
    if (resume(initial, 0)) return die(&state, "initial-resume");
    while (!lpc_empty(&state)) {
        unsigned long message = 0; int signal, event;
        waited = waitpid(-1, &status, __WALL); if (waited < 0) return die(&state, "wait");
        if (WIFEXITED(status)) { if (lpc_exit_process(&state, waited, WEXITSTATUS(status))) return die(&state, "exit"); continue; }
        if (WIFSIGNALED(status) || !WIFSTOPPED(status)) return die(&state, "signaled");
        signal = WSTOPSIG(status); event = status >> 16;
        if (signal == (SIGTRAP | 0x80)) {
            if (syscall_stop(waited, &info)) return die(&state, "syscall-info");
            if (info.op == PTRACE_SYSCALL_INFO_ENTRY && lpc_entry(&state, waited, (long)info.entry.nr, info.entry.args)) return die(&state, "syscall-entry");
            if (info.op == PTRACE_SYSCALL_INFO_EXIT && lpc_exit(&state, waited, (long)info.exit.rval)) return die(&state, "syscall-exit");
            if (info.op != PTRACE_SYSCALL_INFO_ENTRY && info.op != PTRACE_SYSCALL_INFO_EXIT) return die(&state, "syscall-direction");
            if (resume(waited, 0)) return die(&state, "syscall-resume");
        } else if (signal == SIGTRAP && event == PTRACE_EVENT_FORK) {
            if (ptrace(PTRACE_GETEVENTMSG, waited, NULL, &message) || !message || lpc_fork(&state, waited, (pid_t)message) || resume(waited, 0)) return die(&state, "fork-event");
        } else if (signal == SIGTRAP && event == PTRACE_EVENT_EXEC) {
            if (lpc_exec(&state, waited) || resume(waited, 0)) return die(&state, "exec-event");
        } else if (signal == SIGTRAP && (event == PTRACE_EVENT_VFORK || event == PTRACE_EVENT_CLONE)) {
            return die(&state, "unsupported-child-event");
        } else if (signal == SIGSTOP) {
            if (resume(waited, 0)) return die(&state, "child-stop");
        } else if (signal == SIGCHLD) {
            if (resume(waited, SIGCHLD)) return die(&state, "sigchld");
        } else return die(&state, "unexpected-stop");
    }
    if (image("/vulkan/raw.adoc", &after) || !stable(&raw.stat, &after.stat) || memcmp(raw.bytes, after.bytes, sizeof(raw.bytes)) ||
        image("/work/generated/out.adoc", &output) || memcmp(raw.bytes, output.bytes, sizeof(raw.bytes)) ||
        snapshot("/vulkan/raw.adoc", &raw) || snapshot("/work/generated/out.adoc", &output)) return die(&state, "post-exit-snapshot");
    puts("{\"kind\":\"terminal\",\"status\":\"observed-unadmitted\"}");
    return fflush(stdout) ? 77 : 0;
}
