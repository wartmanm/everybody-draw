#!/usr/bin/env sh

rustc src/lib.rs --crate-name rustgl --crate-type staticlib -g --test \
  --out-dir /home/matt/workspace/GLDraw-scala/src/main/rust/target \
  --dep-info /home/matt/workspace/GLDraw-scala/src/main/rust/target/.fingerprint/rustgl-45c2de64c06f60f1/dep-test-lib-rustgl -L /home/matt/workspace/GLDraw-scala/src/main/rust/target -L /home/matt/workspace/GLDraw-scala/src/main/rust/target/deps \
  --extern lua=/home/matt/workspace/GLDraw-scala/src/main/rust/target/deps/liblua-c2043cfdf63c2672.rlib \
  --extern egl=/home/matt/workspace/GLDraw-scala/src/main/rust/target/deps/libegl-f7db8033a2494339.rlib \
  --extern opengles=/home/matt/workspace/GLDraw-scala/src/main/rust/target/deps/libopengles-4135c66f1f24825b.rlib \
  -C link-args="-Wl,--export-dynamic -Wl,--warn-unresolved-symbols"
