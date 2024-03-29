#!/bin/bash

RUSTFLAGS="-C target-feature=+crt-static" \
cargo build --release && strip --strip-unneeded target/release/pokemon_structs && upx --best --lzma target/release/pokemon_structs && target/release/pokemon_structs
