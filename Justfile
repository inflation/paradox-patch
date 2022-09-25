default: install

build:
    cargo pgo test
    cargo pgo optimize

install: build
    cp target/release/steam-dlc ~/.local/bin/steam-dlc
