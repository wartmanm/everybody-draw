use core::prelude::*;
use core::mem;
use jni::{jobject, jclass, jfieldID, JNIEnv, JNINativeMethod};
use android::input::AInputEvent;

use glpoint;
use rustjni::{register_classmethods, jpointer, get_jpointer};

static mut MOTION_CLASS: jclass = 0 as jclass;
static mut MOTIONEVENT_NATIVE_PTR_FIELD: jfieldID = 0 as jfieldID;

unsafe extern "C" fn init_motion_event_handler(env: *mut JNIEnv, _: jobject) -> jobject {
    let (consumer, producer) = glpoint::create_motion_event_handler();
    let (consumer, producer) = (box consumer, box producer);
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let constructor = ((**env).GetMethodID)(env, pairclass, cstr!("<init>"), cstr!("(II)V"));
    let (consumer, producer): (jpointer, jpointer) = (mem::transmute(consumer), mem::transmute(producer));
    ((**env).NewObject)(env, pairclass, constructor, consumer, producer)
}

unsafe extern "C" fn destroy_motion_event_handler(env: *mut JNIEnv, _: jobject, pairobj: jobject) {
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let consumerfield = ((**env).GetFieldID)(env, pairclass, cstr!("consumer"), cstr!("I"));
    let producerfield = ((**env).GetFieldID)(env, pairclass, cstr!("producer"), cstr!("I"));
    let consumer = get_jpointer(env, pairobj, consumerfield);
    let producer = get_jpointer(env, pairobj, producerfield);
    glpoint::destroy_motion_event_handler(mem::transmute(consumer), mem::transmute(producer));
}

unsafe extern "C" fn native_append_motion_event(env: *mut JNIEnv, _: jobject, handler: jpointer, evtobj: jobject) {
    let evtptr = ((**env).GetIntField)(env, evtobj, MOTIONEVENT_NATIVE_PTR_FIELD);
    glpoint::jni_append_motion_event(mem::transmute(handler), evtptr as *const AInputEvent);
}

unsafe extern "C" fn native_pause_motion_event(_: *mut JNIEnv, _: jobject, handler: jpointer) {
    glpoint::jni_pause_motion_event(mem::transmute(handler));
}

pub unsafe fn init(env: *mut JNIEnv) {
    // TODO: use global ref here
    MOTION_CLASS = ((**env).FindClass)(env, cstr!("android/view/MotionEvent"));
    MOTIONEVENT_NATIVE_PTR_FIELD = ((**env).GetFieldID)(env, MOTION_CLASS, cstr!("mNativePtr"), cstr!("I"));
    logi!("got motion classes");

    let producermethods = [
        native_method!("nativeAppendMotionEvent", "(ILandroid/view/MotionEvent;)V", native_append_motion_event),
        native_method!("nativePauseMotionEvent", "(I)V", native_pause_motion_event),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/MotionEventProducer$"), &producermethods);

    let motioneventhandlerstaticmethods = [
        native_method!("init", "()Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;", init_motion_event_handler),
        native_method!("destroy", "(Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;)V", destroy_motion_event_handler),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair$"), &motioneventhandlerstaticmethods);
    logi!("registered motionevent methods!");
}

pub unsafe fn destroy(_: *mut JNIEnv) {

}
