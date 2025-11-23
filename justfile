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

run-tracy:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo run -F profile --no-default-features --release

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


run-flamegraph:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --no-default-features -f staticlink

tracy:
    #TRACY_DPI_SCALE=1.5 tracy
    TRACY_DPI_SCALE=1.5 nix run github:MonaMayrhofer/nixpkgs/tracy-upgrade#tracy
