#!/bin/bash
set -eu

# Adapted from https://github.com/creativcoder/headlines/blob/main/setup_web.sh

rustup target add wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli
cargo update -p wasm-bindgen

cargo install basic-http-server
