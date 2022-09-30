default: install

build:
    cargo build --release

install: build
    cp target/release/steam-dlc ~/.local/bin/steam-dlc
    cargo clean
