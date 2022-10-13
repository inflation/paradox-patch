default: install-cli

release-cli:
    cargo build --release --target aarch64-apple-darwin --bin paradox-patch
    cargo build --release --target x86_64-apple-darwin --bin paradox-patch
    lipo -create -output target/release/paradox-patch \
        target/aarch64-apple-darwin/release/paradox-patch \
        target/x86_64-apple-darwin/release/paradox-patch
    git tag -a v$(VERSION) -m "Release v$(VERSION)"
    gh release create v$(VERSION) target/release/paradox-patch
    cargo clean

install-cli: 
    cargo build --release --target aarch64-apple-darwin
    ditto target/aarch64-apple-darwin/release/paradox-patch ~/.local/bin/paradox-patch
    cargo clean
