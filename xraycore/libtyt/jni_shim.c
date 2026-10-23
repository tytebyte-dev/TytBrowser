// JNI-обёртки над Go-функциями. Класс Kotlin: dev.tytebyte.tytbrowser.core.XrayNative (@JvmStatic external).
#include <jni.h>
#include <stdlib.h>
#include "_cgo_export.h"

static jstring to_jstring(JNIEnv *env, char *s) {
    jstring r = (*env)->NewStringUTF(env, s ? s : "");
    free(s);
    return r;
}

JNIEXPORT jstring JNICALL Java_dev_tytebyte_tytbrowser_core_XrayNative_start(
    JNIEnv *env, jclass cls, jstring config, jint port) {
    const char *cfg = (*env)->GetStringUTFChars(env, config, NULL);
    char *res = TytStart((char *)cfg, (int)port);
    (*env)->ReleaseStringUTFChars(env, config, cfg);
    return to_jstring(env, res);
}

JNIEXPORT void JNICALL Java_dev_tytebyte_tytbrowser_core_XrayNative_stop(JNIEnv *env, jclass cls) {
    TytStop();
}

JNIEXPORT jint JNICALL Java_dev_tytebyte_tytbrowser_core_XrayNative_port(JNIEnv *env, jclass cls) {
    return (jint)TytPort();
}

JNIEXPORT jstring JNICALL Java_dev_tytebyte_tytbrowser_core_XrayNative_log(JNIEnv *env, jclass cls) {
    return to_jstring(env, TytLog());
}
