#include <jni.h>
#define NULL 0

jint JNI_OnLoad(JavaVM* vm, void* reserved);
// make sure rust's jni_onload gets linked in
// TODO: find a better way
static void ensure_jni_onload_exists() {
  JNI_OnLoad(NULL, NULL);
}
