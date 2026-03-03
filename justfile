set windows-shell := ["powershell.exe"]
export RUST_BACKTRACE := "1"

@just:
    just --list

build:
    cargo build -r

run *args:
    cargo run -r -- {{args}}

lint:
    cargo clippy --all --tests -- -D warnings

fix:
    cargo clippy --all --tests --fix

format:
    cargo fmt --all

check:
    cargo check --all --tests

test:
    cargo test --all -- --nocapture
