RUST_FLAGS := --target=${TARGET_TRIPLE} --out-dir=${OUT_DIR} -g
PLATFORM_PATH := $(ANDROID_NDK_ROOT)/platforms/${PLATFORM_NAME}/arch-arm

RUST_FILES := copyshader.rs glcommon.rs glinit.rs glpoint.rs\
						 motionevent.rs pointshader.rs point.rs gltexture.rs\
						 rollingaverage.rs macros.rs matrix.rs eglinit.rs\
						 drawevent.rs glstore.rs

RUST_LIBS := opengles egl

OUT_FILES := librustgl.a

BINDINGS := android/input android/log

LIB_FILES := $(shell echo "$(RUST_LIBS)" | sed -E 's!\S+!deps/rust-\0/lib\0.rlib!g')
BINDING_RUSTFILES := $(BINDINGS:=.rs)

#TODO: autogenerate binding dirs and mod.rs files
$(BINDING_RUSTFILES): %.rs: $(PLATFORM_PATH)/usr/include/%.h
	bindgen $< \
		-I $(PLATFORM_PATH)/usr/include \
		-I $(ANDROID_NDK_ROOT)/toolchains/arm-linux-androideabi-4.6/prebuilt/linux-x86_64/lib/gcc/arm-linux-androideabi/4.6/include/ \
		-builtins \
		> $@

$(LIB_FILES):
	RUSTFLAGS="--opt-level=3 --target arm-linux-androideabi\
						-C linker=$(ANDROID_NDK_ROOT)/toolchains/arm-linux-androideabi-4.6/prebuilt/linux-x86_64/bin/arm-linux-androideabi-gcc" \
						make -C $(dir $@)

$(LIB_FILES:=.clean):
	make -C $(dir $@) clean

$(OUT_FILES): lib.rs $(LIB_FILES) $(BINDING_RUSTFILES) $(RUST_FILES)
	rustc $< $(RUST_FLAGS) $(patsubst %, -L %, $(dir $(LIB_FILES)))

.PHONY: all
all: $(OUT_FILES)

.PHONY: clean
clean: $(LIB_FILES:=.clean)
	rm -f $(OUT_FILES) $(BINDING_RUSTFILES)
	
