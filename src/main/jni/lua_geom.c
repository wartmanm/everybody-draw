#include <android/log.h>

void loglua(char *message) {
  __android_log_print(ANDROID_LOG_INFO, "luascript", "%s", message);
}
