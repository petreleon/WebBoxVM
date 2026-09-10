#ifndef WEBBOXVM_LINEAGE_PTRACE_STATE_H
#define WEBBOXVM_LINEAGE_PTRACE_STATE_H

#include <stdint.h>
#include <stdio.h>
#include <sys/types.h>

#define LPC_PATH 128
#define LPC_FDS 8
#define LPC_PROCS 8

enum lpc_call { LPC_NONE, LPC_OPEN, LPC_READ, LPC_WRITE, LPC_CLOSE, LPC_RENAME };

struct lpc_fd {
    int used, number, mode;
    char path[LPC_PATH];
};

struct lpc_pending {
    enum lpc_call kind;
    long fd, flags;
    char old[LPC_PATH], new[LPC_PATH];
};

struct lpc_process {
    int used;
    pid_t pid, parent;
    struct lpc_pending pending;
    struct lpc_fd fds[LPC_FDS];
};

struct lpc_state {
    FILE *out;
    struct lpc_process processes[LPC_PROCS];
};

void lpc_init(struct lpc_state *, FILE *);
int lpc_start(struct lpc_state *, pid_t, pid_t);
int lpc_fork(struct lpc_state *, pid_t, pid_t);
int lpc_entry(struct lpc_state *, pid_t, long, const uint64_t *);
int lpc_exit(struct lpc_state *, pid_t, long);
int lpc_exec(struct lpc_state *, pid_t);
int lpc_exit_process(struct lpc_state *, pid_t, int);
int lpc_empty(const struct lpc_state *);
void lpc_kill(const struct lpc_state *);

#endif
