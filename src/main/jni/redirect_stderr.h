#include <pthread.h>
#include <stdio.h>

struct readpipes {
  int pipe_stdout;
  int pipe_stderr;
  int pipe_done;
};

struct selectstream {
  int fd;
  FILE* stream;
  int loglevel;
};

struct stdout_forwarder {
  pthread_t threadid;
  int stdout_pipes[2];
  int stderr_pipes[2];
  int done_pipes[2];
  struct readpipes* thread_pipes;
};

int begin_forwarding(struct stdout_forwarder* f);
int end_forwarding(struct stdout_forwarder* f);

