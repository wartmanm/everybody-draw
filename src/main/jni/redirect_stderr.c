#include <pthread.h>
#include <sys/select.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <errno.h>

#include <android/log.h>

#include "redirect_stderr.h"

#define LOGI(...) ((void)__android_log_print(ANDROID_LOG_INFO, "redirector", __VA_ARGS__))
#define LOGERR() __android_log_print(ANDROID_LOG_ERROR, "redirector", "error in %s at %s:%d :: %s", __func__, __FILE__, __LINE__, strerror(errno))
/*static void logi(char* msg) {*/
  /*__android_log_write(ANDROID_LOG_INFO, "redirector", msg);*/
/*}*/

static int pipe_or_err(int fds[2], int newfd) {
  int pipes[2];
  if (0 != pipe(pipes)) {
    LOGERR();
    return -1;
  }
  int dupedfd = dup2(pipes[1], newfd);
  if (-1 == dupedfd) {
    LOGERR();
    close(pipes[0]);
    close(pipes[1]);
    return -1;
  }
  if (-1 == close(pipes[1])) {
    LOGERR();
    close(pipes[0]);
    close(dupedfd);
    return -1;
  }
  fds[0] = pipes[0];
  fds[1] = dupedfd;
  return 0;
}

static int selectstream_create(struct selectstream* s, int fd) {
  FILE* stream = fdopen(fd, "r");
  if (stream == NULL) {
    return -1;
  }
  (*s) = (struct selectstream) {
    .fd = fd,
    .stream = stream,
  };
  return 0;
}

static int selectstream_copyline(struct selectstream* s, char* buffer, int size) {
  if (NULL == fgets(buffer, size, s -> stream)) {
    return -1;
  }
  __android_log_write(s -> loglevel, "glinit", buffer);
  return 0;
}

static void* perform_read(void* args) {
  LOGI("in listener thread");
  struct readpipes* readpipe = (struct readpipes*) args;
  fd_set readfds;
  FD_ZERO(&readfds);
  struct selectstream streams[2];
  if (selectstream_create(&streams[0], readpipe -> pipe_stdout) == -1) {
    LOGERR();
    goto done;
  }
  if (selectstream_create(&streams[1], readpipe -> pipe_stderr) == -1) {
    LOGERR();
    goto cleanup_pstdout;
  }
  int* fds = (int*) (void*) readpipe;
  int max = 0;
  for (int i = 0; i < 3; i++) {
    max = max > fds[i] ? max : fds[i];
  }
  while (1) {
    FD_ZERO(&readfds);
    for (int i = 0; i < 3; i++) {
      FD_SET(fds[i], &readfds);
    }
    char buffer[4096];
    LOGI("listening");
    int selected = select(max, &readfds, NULL, NULL, NULL);
    if (selected == -1) {
      LOGERR();
      goto cleanup_pstderr;
    }
    for (unsigned int i = 0; selected > 0 && i < sizeof streams / sizeof streams[0]; i++) {
      struct selectstream* stream = &streams[i];
      if (FD_ISSET(stream -> fd, &readfds)) {
        selected -= 1;
        if (selectstream_copyline(stream, buffer, sizeof buffer)) {
          goto cleanup_pstderr;
        }
      }
    }
    if (selected > 0) { // only the "done" pipe is left, this must be it
      break;
    }
  }
cleanup_pstderr:
  fclose(streams[1].stream);
cleanup_pstdout:
  fclose(streams[0].stream);
done:
  LOGI("ended listener thread");
  return NULL;
}

int begin_forwarding(struct stdout_forwarder* f) {
  pthread_attr_t logthread_attrs;
  pthread_attr_init(&logthread_attrs);

  // The fgetpos() calls matter.  No idea why.
  fpos_t pos;
  fflush(stdout);
  fgetpos(stdout, &pos);
  fflush(stderr);
  fgetpos(stderr, &pos);
  if (pipe_or_err(f -> stdout_pipes, STDOUT_FILENO) == -1) {
    goto abort_done;
    return -1;
  }
  if (pipe_or_err(f -> stderr_pipes, STDOUT_FILENO) == -1) {
    goto abort_stdout_pipes;
  }
  if (-1 == pipe(f -> done_pipes)) {
    goto abort_stderr_pipes;
  }
  struct readpipes* thread_pipes = malloc(sizeof (struct readpipes));
  thread_pipes -> pipe_stdout = f -> stdout_pipes[0];
  thread_pipes -> pipe_stderr = f -> stderr_pipes[0];
  thread_pipes -> pipe_done = f -> done_pipes[0];
  f -> thread_pipes = thread_pipes;

  if (pthread_create(&f->threadid, &logthread_attrs, perform_read, thread_pipes) != 0) {
    goto abort_thread_pipes;
  }
  pthread_attr_destroy(&logthread_attrs);
  return 0;

abort_thread_pipes:
  free(thread_pipes);
  close(f -> done_pipes[0]);
  close(f -> done_pipes[1]);
abort_stderr_pipes:
  close(f -> stderr_pipes[0]);
  close(f -> stderr_pipes[1]);
abort_stdout_pipes:
  close(f -> stdout_pipes[0]);
  close(f -> stdout_pipes[1]);
abort_done:
  return -1;
}

int end_forwarding(struct stdout_forwarder* f) {
  write(f->done_pipes[1], ".", 1);
  void *result = NULL;
  pthread_join(f->threadid, result);
  free(result);
  free(f->thread_pipes);
  close(f -> done_pipes[0]);
  close(f -> done_pipes[1]);

  close(f -> stderr_pipes[0]);
  close(f -> stderr_pipes[1]);

  close(f -> stdout_pipes[0]);
  close(f -> stdout_pipes[1]);
  return 0;
}



