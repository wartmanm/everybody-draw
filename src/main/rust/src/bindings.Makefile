PLATFORM_PATH := $(ANDROID_NDK_ROOT)/platforms/${PLATFORM_NAME}/arch-arm

BINDINGS := android/input android/log

BINDING_RUSTFILES := $(BINDINGS:=.rs)

#TODO: autogenerate binding dirs and mod.rs files
$(BINDING_RUSTFILES): %.rs: $(PLATFORM_PATH)/usr/include/%.h
	bindgen $< \
		-I $(PLATFORM_PATH)/usr/include \
		-I $(ANDROID_NDK_ROOT)/toolchains/arm-linux-androideabi-4.6/prebuilt/linux-x86_64/lib/gcc/arm-linux-androideabi/4.6/include/ \
		-builtins \
		> $@

bindings: $(BINDING_RUSTFILES)

all: bindings

.PHONY: clean
clean: $(LIB_FILES:=.clean)
	rm -f $(BINDING_RUSTFILES)
	
