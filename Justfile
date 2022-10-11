default: install

build:
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    lipo -create -output target/release/paradox-patch \
        target/aarch64-apple-darwin/release/paradox-patch \
        target/x86_64-apple-darwin/release/paradox-patch

install: build
    ditto target/aarch64-apple-darwin/release/paradox-patch ~/.local/bin/paradox-patch
