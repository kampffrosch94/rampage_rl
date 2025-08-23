default:
    @just --list

export RUST_BACKTRACE := "1"

[working-directory: 'worker']
watch:
    cargo-watch -x build --clear  -d 0.05

[working-directory: 'app']
run:
    cargo run

[working-directory: 'app']
run-gdb:
    cargo build
    gdb ../target/debug/app

[working-directory: 'app']
run-lldb:
    cargo build -Zbuild-std
    lldb ../target/debug/app

run-static:
    cargo run -F staticlink --no-default-features

web-build:
    cargo build --target wasm32-unknown-unknown --release -F staticlink --no-default-features


web-serve:
    @just web-build
    cp -r assets/ target/wasm32-unknown-unknown/release
    cp web/* target/wasm32-unknown-unknown/release
    @just web-server

[working-directory: 'target/wasm32-unknown-unknown/release']
web-server:
    python -m http.server 8000


profile:
    cargo flamegraph --no-default-features -f staticlink
