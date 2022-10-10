default: install

build:
    cargo build --release

install: build
    ditto target/release/steam-dlc ~/.local/bin/steam-dlc
    cargo clean
