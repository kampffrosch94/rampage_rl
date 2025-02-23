default:
    @just --list

watch:
    cd worker && cargo-watch -x build --clear  -d 0.05
web-build:
    cargo build --target wasm32-unknown-unknown --release -F staticlink --no-default-features
