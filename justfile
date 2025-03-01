default:
    @just --list

[working-directory: 'worker']
watch:
    cargo-watch -x build --clear  -d 0.05

[working-directory: 'app']
run:
    RUST_BACKTRACE=1 cargo run

run-static:
    cargo run -F staticlink --no-default-features

web-build:
    cargo build --target wasm32-unknown-unknown --release -F staticlink --no-default-features

web-deploy:
    @just web-build
    @just -f ~/data/Programming/static/roguelike_template_test/justfile deploy
