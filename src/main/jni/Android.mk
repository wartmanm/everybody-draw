# Copyright (C) 2009 The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE    := rustgl
LOCAL_ARM_MODE := arm
LOCAL_SRC_FILES := ../rust/target/arm-linux-androideabi/librustgl-45c2de64c06f60f1.a

include $(PREBUILT_STATIC_LIBRARY)

include $(CLEAR_VARS)

LOCAL_MODULE := luajit
LOCAL_ARM_MODE := arm
LOCAL_SRC_FILES := ../../../lib/libluajit.a

include $(PREBUILT_STATIC_LIBRARY)

include $(CLEAR_VARS)

LOCAL_MODULE    := gl-stuff
LOCAL_ARM_MODE := arm
LOCAL_SRC_FILES := hello-jni.c unwind.c lua_geom.c

LOCAL_CFLAGS := -std=c99 -Wall -Wextra -Wno-unused -Werror -g -fexceptions
LOCAL_LDFLAGS := -L/opt/android-ndk/android-ndk-r9b/sources/cxx-stl/gnu-libstdc++/4.6/libs/armeabi -z muldefs
LOCAL_LDLIBS    := -lGLESv2 -ldl -llog -lc -lEGL -landroid -ljnigraphics
LOCAL_STATIC_LIBRARIES := rustgl luajit
#LOCAL_SHARED_LIBRARIES := luajit
# LOCAL_CPP_FEATURES += exceptions
# LOCAL_CPP_FEATURES += rtti

include $(BUILD_SHARED_LIBRARY)
